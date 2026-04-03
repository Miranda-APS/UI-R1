"""
Loop Supervisor per l'Agente Semantico di Prometeo.
Gestisce il ciclo di interazione con Qwen via Ollama.
"""
import json
import time
import sys
import io
from pathlib import Path
from typing import Optional
from dataclasses import dataclass

# Fix encoding Windows
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

# Aggiungi la directory agent al path per import assoluti
sys.path.insert(0, str(Path(__file__).parent))

from ollama_client import OllamaClient, TokenLimiter
from prometeo_bridge import PrometeoBridge
from logger import OpinionLogger, OpinionSummarizer
from tools import ToolRegistry


@dataclass
class LoopConfig:
    """Configurazione del loop."""
    max_iterations: int = 50
    token_budget_per_turn: int = 2048
    temperature: float = 0.3
    pause_every: int = 10
    pause_duration: int = 5
    max_errors: int = 3
    max_no_improvement: int = 5
    log_opinions_every: int = 5


class PrometeoAgent:
    """
    Agente semantico principale.
    Orchestra Ollama ↔ Bridge ↔ Tool Registry.
    """
    
    def __init__(
        self,
        config: LoopConfig,
        task_file: Optional[str] = None,
    ):
        self.config = config
        self.task_file = task_file
        
        # Componenti
        self.ollama = OllamaClient(
            model="qwen2.5:7b-instruct",
            temperature=config.temperature,
            num_predict=config.token_budget_per_turn
        )
        self.bridge = PrometeoBridge()
        self.logger = OpinionLogger()
        self.summarizer = OpinionSummarizer(self.logger)
        self.tools = ToolRegistry(self.bridge, self.logger)
        self.token_limiter = TokenLimiter()
        
        # Stato loop
        self.iteration = 0
        self.error_count = 0
        self.no_improvement_count = 0
        self.consecutive_tool_calls = 0
        self.messages: list[dict] = []
        
        # Carica system prompt
        self.system_prompt = self._load_system_prompt()
        
        # Carica task se specificato
        self.task_config = self._load_task() if task_file else None
        
    def _load_system_prompt(self) -> str:
        """Carica e personalizza il system prompt."""
        prompt_path = Path("tools/steering/system_prompt.md")
        if not prompt_path.exists():
            return "Sei un agente semantico per Prometeo."
            
        template = prompt_path.read_text(encoding='utf-8')
        
        # Inserisci stato attuale
        stats = self.bridge.get_stats()
        system_state = (
            f"{stats.word_count} parole, "
            f"{stats.kg_edges} archi KG, "
            f"{len(stats.isolated_words)} nodi isolati"
        )
        
        template = template.replace("{{SYSTEM_STATE}}", system_state)
        template = template.replace("{{CURRENT_TASK}}", self.task_file or "explorazione_libera")
        template = template.replace("{{ITERATION}}", str(self.iteration))
        
        return template
    
    def _load_task(self) -> Optional[dict]:
        """Carica configurazione task da JSON."""
        path = Path(self.task_file)
        if not path.exists():
            print(f"[WARN] Task file non trovato: {path}")
            return None
        return json.loads(path.read_text(encoding='utf-8'))
    
    def _get_tools_schema(self) -> list:
        """Carica gli schemi tool per Ollama."""
        schema_path = Path("tools/steering/schemas/tool_schemas.json")
        if schema_path.exists():
            return json.loads(schema_path.read_text())['tools']
        return []
    
    def _update_system_prompt(self):
        """Aggiorna il system prompt con stato corrente."""
        self.system_prompt = self._load_system_prompt()
        # Aggiungi riepilogo opinioni recenti
        recent = self.summarizer.summarize_for_context(max_chars=1000)
        self.system_prompt += f"\n\n{recent}"
    
    def _check_circuit_breaker(self) -> bool:
        """Verifica se fermare il loop per condizioni critiche."""
        if self.error_count >= self.config.max_errors:
            self.logger.log_opinion(
                category="meta",
                content=f"CIRCUIT BREAKER: Troppi errori ({self.error_count}). Fermato.",
            )
            return True
            
        if self.no_improvement_count >= self.config.max_no_improvement:
            self.logger.log_opinion(
                category="meta",
                content=f"CIRCUIT BREAKER: Nessun miglioramento per {self.no_improvement_count} iterazioni.",
            )
            return True
            
        return False
    
    def _prepare_initial_message(self) -> str:
        """Prepara il messaggio iniziale per Qwen."""
        stats = self.bridge.get_stats()
        
        msg = f"""Inizia l'esplorazione del campo topologico di Prometeo.

STATO ATTUALE:
- {stats.word_count} parole nel lessico
- {stats.kg_edges} archi nel Knowledge Graph  
- {len(stats.isolated_words)} nodi isolati rilevati
- {len(stats.densest_nodes)} nodi ad alta connessione

ISOLATI CAMPIONE: {', '.join(stats.isolated_words[:10])}

Il tuo compito è esplorare, analizzare e migliorare le connessioni semantiche.
Inizia con un'analisi del campo (analyze_field) per identificare i problemi più critici.
Poi procedi sistematicamente con query, proposte e validazioni.

Ricorda: logga le tue opinioni con log_opinion, sii conservativo nelle modifiche,
e verifica sempre la coerenza frattale prima di committare."""

        if self.task_config:
            msg += f"\n\nTASK CONFIGURATO: {self.task_config.get('name', 'unknown')}"
            msg += f"\nStrategia: {self.task_config.get('strategy', {}).get('type', 'default')}"
            
        return msg
    
    def run(self):
        """Esegue il loop principale."""
        print("=" * 60)
        print("🧠 PROMETEO SEMANTIC AGENT")
        print("=" * 60)
        
        # Health check Ollama
        print("\n[1] Verifica Ollama...")
        if not self.ollama.health_check():
            print("❌ Ollama non raggiungibile o modello non caricato")
            print("   Avvia: ollama run qwen3:9b")
            sys.exit(1)
        print("✅ Ollama pronto")
        
        # Log iniziale
        self.logger.log_opinion(
            category="meta",
            content=f"Agente avviato. Task: {self.task_file or 'explorazione_libera'}. "
                    f"Max iterazioni: {self.config.max_iterations}",
        )
        
        # Inizializza conversazione
        self.messages = [
            {"role": "user", "content": self._prepare_initial_message()}
        ]
        
        # Loop principale
        for i in range(self.config.max_iterations):
            self.iteration = i + 1
            print(f"\n{'─' * 60}")
            print(f"🔁 ITERAZIONE {self.iteration}/{self.config.max_iterations}")
            print(f"💰 Token budget: {self.token_limiter.get_remaining()}")
            
            # Check circuit breaker
            if self._check_circuit_breaker():
                break
                
            # Check token budget
            if not self.token_limiter.check_budget(1000):
                print("⏸️ Token budget esaurito, pausa...")
                time.sleep(60)
                continue
            
            try:
                # Aggiorna system prompt con opinioni recenti
                if self.iteration % self.config.log_opinions_every == 0:
                    self._update_system_prompt()
                
                # Chiama Ollama
                print("🤔 Qwen sta ragionando...")
                response = self.ollama.chat(
                    messages=self.messages,
                    tools=self._get_tools_schema(),
                    system=self.system_prompt
                )
                
                self.token_limiter.consume(
                    response.usage.get('prompt_tokens', 0) + 
                    response.usage.get('completion_tokens', 0)
                )
                
                # Gestisci risposta
                if response.tool_call:
                    # Esegui tool
                    print(f"🔧 Tool call: {response.tool_call.name}")
                    result = self.tools.execute(
                        response.tool_call.name,
                        response.tool_call.arguments
                    )
                    
                    # Aggiorna conversazione
                    self.messages.append({
                        "role": "assistant",
                        "content": response.content or f"Uso {response.tool_call.name}"
                    })
                    self.messages.append({
                        "role": "tool",
                        "content": json.dumps({
                            "tool": response.tool_call.name,
                            "result": result.data,
                            "message": result.message
                        })
                    })
                    
                    # Stato
                    print(f"   Status: {'✅' if result.success else '❌'} {result.message[:80]}")
                    
                    # Aggiorna contatori
                    if result.success:
                        self.no_improvement_count = 0
                        self.consecutive_tool_calls += 1
                    else:
                        self.error_count += 1
                        
                    # Suggerimento prossimo tool
                    if result.suggested_next:
                        print(f"   ➡️ Suggerito: {result.suggested_next}")
                        
                else:
                    # Risposta testuale senza tool
                    print(f"💬 Qwen: {response.content[:100]}...")
                    self.messages.append({
                        "role": "assistant",
                        "content": response.content
                    })
                    self.consecutive_tool_calls = 0
                    
                    # Se troppe risposte senza tool, forza analisi
                    if self.consecutive_tool_calls == 0 and self.iteration % 5 == 0:
                        self.messages.append({
                            "role": "user", 
                            "content": "Procedi con un tool call. Analizza il campo o proponi connessioni."
                        })
                
                # Pausa periodica
                if self.iteration % self.config.pause_every == 0:
                    print(f"⏸️ Pausa di {self.config.pause_duration}s...")
                    time.sleep(self.config.pause_duration)
                    
            except Exception as e:
                self.error_count += 1
                self.logger.log_error(str(e), f"Iterazione {self.iteration}")
                print(f"❌ Errore: {e}")
                time.sleep(5)
        
        # Cleanup
        self._shutdown()
    
    def _shutdown(self):
        """Termina la sessione."""
        print(f"\n{'=' * 60}")
        print("🏁 SESSIONE TERMINATA")
        print(f"{'=' * 60}")
        
        # Salva KG se ci sono modifiche
        if self.tools._pending_changes:
            print(f"\n💾 Salvataggio {len(self.tools._pending_changes)} modifiche pendenti...")
            self.tools.commit_changes(batch_id="final", dry_run=False)
        
        # Log finale
        self.logger.log_opinion(
            category="meta",
            content=f"Sessione terminata dopo {self.iteration} iterazioni. "
                    f"Errori: {self.error_count}",
        )
        self.logger.save_session()
        
        print(f"\n📁 Log salvati in: tools/logs/")
        print(f"📊 Iterazioni completate: {self.iteration}")


def main():
    """Entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Prometeo Semantic Agent")
    parser.add_argument('--task', '-t', help='File task JSON da tools/steering/tasks/')
    parser.add_argument('--iterations', '-i', type=int, default=50, help='Max iterazioni')
    parser.add_argument('--temp', type=float, default=0.3, help='Temperatura Qwen')
    parser.add_argument('--dry-run', action='store_true', help='Simula senza salvare')
    
    args = parser.parse_args()
    
    config = LoopConfig(
        max_iterations=args.iterations,
        temperature=args.temp
    )
    
    agent = PrometeoAgent(
        config=config,
        task_file=args.task
    )
    
    try:
        agent.run()
    except KeyboardInterrupt:
        print("\n\n⚠️ Interrotto dall'utente")
        agent._shutdown()


if __name__ == "__main__":
    main()
