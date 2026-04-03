"""
Implementazione dei tool che l'agente può chiamare.
Ogni funzione corrisponde a un tool nello schema JSON.
"""
import time
from typing import Optional
from dataclasses import dataclass

try:
    from prometeo_bridge import PrometeoBridge
    from logger import OpinionLogger
except ImportError:
    from .prometeo_bridge import PrometeoBridge
    from .logger import OpinionLogger


@dataclass
class ToolResult:
    """Risultato di un tool call."""
    success: bool
    data: dict
    message: str
    suggested_next: Optional[str] = None


class ToolRegistry:
    """
    Registro di tutti i tool disponibili.
    Mappa i nomi dei tool alle funzioni Python.
    """
    
    def __init__(self, bridge: PrometeoBridge, logger: OpinionLogger):
        self.bridge = bridge
        self.logger = logger
        self._pending_changes: list[dict] = []  # Batch di modifiche
        
        # Mappa nome → funzione
        self._tools = {
            'analyze_field': self.analyze_field,
            'query_concept': self.query_concept,
            'propose_connection': self.propose_connection,
            'check_fractal_consistency': self.check_fractal_consistency,
            'find_analogies': self.find_analogies,
            'log_opinion': self.log_opinion,
            'commit_changes': self.commit_changes,
            'pause_loop': self.pause_loop,
        }
        
    def execute(self, tool_name: str, arguments: dict) -> ToolResult:
        """Esegue un tool dato il nome e gli argomenti."""
        if tool_name not in self._tools:
            return ToolResult(
                success=False,
                data={},
                message=f"Tool sconosciuto: {tool_name}"
            )
            
        try:
            return self._tools[tool_name](**arguments)
        except Exception as e:
            return ToolResult(
                success=False,
                data={},
                message=f"Errore esecuzione {tool_name}: {e}"
            )
    
    # ═══════════════════════════════════════════════════════════════════
    # Tool Implementations
    # ═══════════════════════════════════════════════════════════════════
    
    def analyze_field(self, focus: str, depth: int) -> ToolResult:
        """Analizza lo stato del campo topologico."""
        stats = self.bridge.get_stats()
        
        result_data = {
            'stats': stats.to_dict(),
            'focus': focus,
            'depth': depth
        }
        
        message_parts = [
            f"Campo: {stats.word_count} parole, {stats.kg_edges} archi KG",
        ]
        
        if focus == 'isolated_nodes':
            isolated = self.bridge.find_isolated_nodes(min_connections=2)
            result_data['isolated_nodes'] = isolated[:20]
            message_parts.append(f"Trovati {len(isolated)} nodi isolati")
            
            # Log opinion automatica
            if isolated:
                self.logger.log_opinion(
                    content=f"Rilevati {len(isolated)} nodi con <2 connessioni. "
                            f"Esempi: {', '.join(isolated[:5])}",
                    category="observation",
                    related_concepts=isolated[:5]
                )
                
        elif focus == 'semantic_gaps':
            gaps = self.bridge.find_semantic_gaps()
            result_data['gaps'] = gaps
            message_parts.append(f"Trovati {len(gaps)} potenziali gap semantici")
            
        elif focus == 'dense_clusters':
            dense = stats.densest_nodes[:10]
            result_data['densest'] = dense
            message_parts.append(f"Nodi più densi: {dense[:3]}")
            
        return ToolResult(
            success=True,
            data=result_data,
            message='; '.join(message_parts),
            suggested_next='query_concept' if focus == 'isolated_nodes' else 'propose_connection'
        )
    
    def query_concept(self, concept: str, relation_type: str = "Any") -> ToolResult:
        """Interroga un concetto specifico."""
        info = self.bridge.query_concept(concept, relation_type)
        
        if not info['exists']:
            return ToolResult(
                success=False,
                data=info,
                message=f"Concetto '{concept}' non trovato nel campo"
            )
            
        parts = [f"'{concept}' esiste nel campo."]
        
        if info['outgoing']:
            parts.append(f"{len(info['outgoing'])} relazioni uscenti")
        if info['incoming']:
            parts.append(f"{len(info['incoming'])} relazioni entranti")
        if info['similar']:
            parts.append(f"Simile a: {', '.join(info['similar'][:5])}")
        if info['is_a']:
            parts.append(f"È un: {', '.join(info['is_a'][:3])}")
            
        # Opinione su nodi sottoconnessi
        total_conn = len(info['outgoing']) + len(info['incoming'])
        if total_conn < 2:
            self.logger.log_opinion(
                content=f"'{concept}' è sottoconnesso ({total_conn} archi). "
                        f"Considerare espansione semantica.",
                category="concern",
                related_concepts=[concept]
            )
            
        return ToolResult(
            success=True,
            data=info,
            message='; '.join(parts),
            suggested_next='propose_connection' if total_conn < 3 else None
        )
    
    def propose_connection(self, subject: str, relation: str, obj: str,
                           confidence: float, reasoning: str) -> ToolResult:
        """Propone una nuova connessione."""
        # Validazione base
        if confidence < 0.5:
            self.logger.log_proposal(subject, relation, obj, confidence, False)
            return ToolResult(
                success=False,
                data={},
                message=f"Confidenza troppo bassa ({confidence})",
            )
            
        # Verifica esistenza parole
        subj_info = self.bridge.query_concept(subject)
        obj_info = self.bridge.query_concept(obj)
        
        if not subj_info['exists']:
            return ToolResult(
                success=False,
                data={},
                message=f"Soggetto '{subject}' non esiste"
            )
        if not obj_info['exists']:
            return ToolResult(
                success=False,
                data={},
                message=f"Oggetto '{obj}' non esiste"
            )
            
        # Aggiungi al batch
        proposal = {
            'subject': subject,
            'relation': relation,
            'object': obj,
            'confidence': confidence,
            'reasoning': reasoning
        }
        self._pending_changes.append(proposal)
        
        # Log
        self.logger.log_opinion(
            content=f"Proposta connessione: {subject} --[{relation}]--> {obj}. "
                    f"Ragionamento: {reasoning}",
            category="suggestion",
            related_concepts=[subject, obj],
            confidence=confidence
        )
        
        return ToolResult(
            success=True,
            data={'proposal': proposal, 'batch_size': len(self._pending_changes)},
            message=f"Proposta accettata nel batch ({len(self._pending_changes)} totali)",
            suggested_next='propose_connection' if len(self._pending_changes) < 10 else 'commit_changes'
        )
    
    def check_fractal_consistency(self, concept: str, expected_fractal: Optional[int] = None) -> ToolResult:
        """Verifica coerenza frattale."""
        result = self.bridge.check_fractal_consistency(concept)
        
        if result['warnings']:
            self.logger.log_opinion(
                content=f"Coerenza frattale di '{concept}': " + 
                        '; '.join(result['warnings']),
                category="concern",
                related_concepts=[concept]
            )
            
        return ToolResult(
            success=True,
            data=result,
            message=f"Coerenza: {result['coherence_score']:.2f}. " + 
                    (' '.join(result['warnings']) if result['warnings'] else 'Nessun problema rilevato')
        )
    
    def find_analogies(self, concept_a: str, concept_b: str, concept_c: str,
                       max_suggestions: int = 3) -> ToolResult:
        """Trova analogie: A:B come C:?"""
        # Analizza relazioni tra A e B
        info_a = self.bridge.query_concept(concept_a)
        info_b = self.bridge.query_concept(concept_b)
        info_c = self.bridge.query_concept(concept_c)
        
        # Euristica: trova concetti che stanno a C come B sta ad A
        suggestions = []
        
        # Se A IsA X e B IsA X, allora cerchiamo cosa è IsA dei genitori di C
        common_parents = set(info_a['is_a']) & set(info_b['is_a'])
        if common_parents:
            # Cerca concetti che hanno relazioni simili a C
            for parent in info_c['is_a']:
                # Trova siblings di C
                siblings = self.bridge.query_concept(parent)
                # Cerca nei incoming chi ha IsA questo parent
                for edge in siblings.get('incoming', []):
                    if edge['relation'] == 'IsA' and edge['subject'] != concept_c:
                        suggestions.append(edge['subject'])
                        
        suggestions = suggestions[:max_suggestions]
        
        return ToolResult(
            success=True,
            data={
                'pattern': f"{concept_a}:{concept_b}::{concept_c}:?",
                'suggestions': suggestions
            },
            message=f"Analogie suggerite per {concept_c}: {', '.join(suggestions) if suggestions else 'Nessuna trovata'}"
        )
    
    def log_opinion(self, category: str, content: str, 
                    related_concepts: Optional[list] = None) -> ToolResult:
        """Registra un'opinione."""
        self.logger.log_opinion(
            category=category,
            content=content,
            related_concepts=related_concepts or []
        )
        return ToolResult(
            success=True,
            data={'logged': True},
            message=f"Opinione registrata: [{category}] {content[:50]}..."
        )
    
    def commit_changes(self, batch_id: str, dry_run: bool = True) -> ToolResult:
        """Applica le modifiche pendenti."""
        if not self._pending_changes:
            return ToolResult(
                success=True,
                data={},
                message="Nessuna modifica da applicare"
            )
            
        results = {
            'dry_run': dry_run,
            'total': len(self._pending_changes),
            'applied': 0,
            'failed': 0,
            'details': []
        }
        
        for change in self._pending_changes:
            if dry_run:
                results['details'].append({
                    'status': 'simulated',
                    'change': change
                })
                results['applied'] += 1
            else:
                success = self.bridge.add_proposed_edge(
                    change['subject'],
                    change['relation'],
                    change['object'],
                    change['confidence'],
                    change['reasoning']
                )
                if success:
                    results['applied'] += 1
                    self.logger.log_proposal(
                        change['subject'], change['relation'], change['object'],
                        change['confidence'], True
                    )
                else:
                    results['failed'] += 1
                    self.logger.log_proposal(
                        change['subject'], change['relation'], change['object'],
                        change['confidence'], False
                    )
                    
        if not dry_run:
            self.bridge.save_kg()
            self._pending_changes = []
            
        return ToolResult(
            success=results['failed'] == 0,
            data=results,
            message=f"{'Simulate' if dry_run else 'Commit'}: {results['applied']}/{results['total']} applicate"
        )
    
    def pause_loop(self, seconds: int, reason: str) -> ToolResult:
        """Mette in pausa il loop."""
        self.logger.log_opinion(
            category="meta",
            content=f"Pausa richiesta: {reason} ({seconds}s)",
        )
        time.sleep(seconds)
        return ToolResult(
            success=True,
            data={'paused_for': seconds},
            message=f"Pausa completata: {reason}"
        )
