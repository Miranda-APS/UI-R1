"""
Client Ollama per Qwen3 con tool calling strutturato.
Ottimizzato per token limitati e risposte rapide.
"""
import json
import requests
from typing import Optional, Callable, Any
from dataclasses import dataclass


@dataclass
class ToolCall:
    name: str
    arguments: dict
    
    @classmethod
    def from_ollama_response(cls, response: dict) -> Optional['ToolCall']:
        """Estrae tool call dalla risposta Ollama."""
        message = response.get('message', {})
        tool_calls = message.get('tool_calls', [])
        
        if not tool_calls:
            return None
            
        tc = tool_calls[0]
        function = tc.get('function', {})
        
        # Gestisci diversi formati di arguments
        args = function.get('arguments', {})
        if isinstance(args, str):
            try:
                args = json.loads(args)
            except json.JSONDecodeError:
                args = {}
                
        return cls(
            name=function.get('name', ''),
            arguments=args
        )


@dataclass
class AgentResponse:
    content: str
    tool_call: Optional[ToolCall]
    usage: dict
    done: bool


class OllamaClient:
    """Client per Ollama con supporto tool calling."""
    
    def __init__(
        self,
        model: str = "qwen2.5:7b-instruct",
        host: str = "http://localhost:11434",
        temperature: float = 0.3,
        num_ctx: int = 8192,
        num_predict: int = 1024,
    ):
        self.model = model
        self.host = host.rstrip('/')
        self.temperature = temperature
        self.num_ctx = num_ctx
        self.num_predict = num_predict
        self.api_url = f"{self.host}/api/chat"
        
        # Sessione per keep-alive
        self.session = requests.Session()
        
    def chat(
        self,
        messages: list[dict],
        tools: Optional[list] = None,
        system: Optional[str] = None,
    ) -> AgentResponse:
        """
        Invia messaggio a Ollama e riceve risposta.
        
        Args:
            messages: Lista di messaggi [{role, content}]
            tools: Lista di tool definitions (JSON Schema)
            system: System prompt opzionale
        """
        payload = {
            "model": self.model,
            "messages": messages,
            "stream": False,
            "options": {
                "temperature": self.temperature,
                "num_ctx": self.num_ctx,
                "num_predict": self.num_predict,
            }
        }
        
        if system:
            payload["system"] = system
            
        if tools:
            payload["tools"] = tools
            
        try:
            response = self.session.post(
                self.api_url,
                json=payload,
                timeout=120
            )
            response.raise_for_status()
            data = response.json()
            
            message = data.get('message', {})
            content = message.get('content', '').strip()
            
            # Estrai tool call
            tool_call = ToolCall.from_ollama_response(data)
            
            # Se c'è tool call, pulisci content (spesso è ridondante)
            if tool_call and content.startswith('<tool_call>'):
                content = ""
                
            return AgentResponse(
                content=content,
                tool_call=tool_call,
                usage={
                    'prompt_tokens': data.get('prompt_eval_count', 0),
                    'completion_tokens': data.get('eval_count', 0),
                },
                done=data.get('done', False)
            )
            
        except requests.exceptions.Timeout:
            raise RuntimeError("Timeout Ollama - modello sovraccarico")
        except requests.exceptions.ConnectionError:
            raise RuntimeError(f"Connessione a Ollama fallita. Ollama è attivo su {self.host}?")
        except Exception as e:
            raise RuntimeError(f"Errore Ollama: {e}")
    
    def health_check(self) -> bool:
        """Verifica se Ollama è attivo e il modello caricato."""
        try:
            resp = self.session.get(f"{self.host}/api/tags", timeout=5)
            resp.raise_for_status()
            models = resp.json().get('models', [])
            return any(self.model in m.get('name', '') for m in models)
        except:
            return False


class TokenLimiter:
    """Gestisce il budget token per iterazioni efficienti."""
    
    def __init__(self, max_tokens_per_minute: int = 10000):
        self.max_tpm = max_tokens_per_minute
        self.used_this_minute = 0
        self.last_reset = 0
        
    def check_budget(self, estimated_tokens: int) -> bool:
        """Verifica se c'è budget sufficiente."""
        import time
        now = time.time()
        
        # Reset ogni minuto
        if now - self.last_reset > 60:
            self.used_this_minute = 0
            self.last_reset = now
            
        return (self.used_this_minute + estimated_tokens) <= self.max_tpm
    
    def consume(self, tokens: int):
        """Registra consumo token."""
        self.used_this_minute += tokens
        
    def get_remaining(self) -> int:
        """Token rimanenti nel minuto corrente."""
        return self.max_tpm - self.used_this_minute
