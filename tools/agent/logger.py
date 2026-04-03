"""
Sistema di logging per le "opinioni" dell'agente.
Ogni riflessione, insight o preoccupazione viene registrata con timestamp e contesto.
"""
import json
import logging
from datetime import datetime
from pathlib import Path
from typing import Optional
from dataclasses import dataclass, asdict


@dataclass
class Opinion:
    """Un'opinione/ osservazione dell'agente."""
    timestamp: str
    iteration: int
    category: str  # observation, concern, insight, suggestion, meta
    content: str
    related_concepts: list[str]
    task_context: str
    confidence: Optional[float] = None
    
    def to_dict(self) -> dict:
        return asdict(self)


class OpinionLogger:
    """
    Logger specializzato per le opinioni dell'agente.
    Crea file JSONL con timestamp per analisi successive.
    """
    
    def __init__(self, log_dir: str = "tools/logs"):
        self.log_dir = Path(log_dir)
        self.log_dir.mkdir(parents=True, exist_ok=True)
        
        # File per sessione
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        self.opinion_file = self.log_dir / f"opinions_{timestamp}.jsonl"
        self.session_file = self.log_dir / f"session_{timestamp}.json"
        
        # Statistiche sessione
        self.stats = {
            'started_at': timestamp,
            'total_opinions': 0,
            'by_category': {},
            'proposals_made': 0,
            'proposals_accepted': 0,
            'errors': []
        }
        
        # Setup anche logging standard
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s | %(levelname)s | %(message)s',
            handlers=[
                logging.FileHandler(self.log_dir / f"agent_{timestamp}.log"),
                logging.StreamHandler()
            ]
        )
        self.logger = logging.getLogger('PrometeoAgent')
        
    def log_opinion(
        self,
        content: str,
        category: str = "observation",
        iteration: int = 0,
        related_concepts: Optional[list] = None,
        task_context: str = "",
        confidence: Optional[float] = None
    ):
        """Registra un'opinione."""
        opinion = Opinion(
            timestamp=datetime.now().isoformat(),
            iteration=iteration,
            category=category,
            content=content,
            related_concepts=related_concepts or [],
            task_context=task_context,
            confidence=confidence
        )
        
        # Salva su JSONL
        with open(self.opinion_file, 'a', encoding='utf-8') as f:
            f.write(json.dumps(opinion.to_dict(), ensure_ascii=False) + '\n')
            
        # Aggiorna stats
        self.stats['total_opinions'] += 1
        self.stats['by_category'][category] = self.stats['by_category'].get(category, 0) + 1
        
        # Log anche su stdout/file
        prefix = {
            'observation': '👁️',
            'concern': '⚠️',
            'insight': '💡',
            'suggestion': '➡️',
            'meta': '🔄'
        }.get(category, '📝')
        
        self.logger.info(f"{prefix} [{category.upper()}] {content[:100]}...")
        
    def log_proposal(self, subject: str, relation: str, obj: str, 
                     confidence: float, accepted: bool):
        """Logga una proposta di connessione."""
        self.stats['proposals_made'] += 1
        if accepted:
            self.stats['proposals_accepted'] += 1
            
        status = "✅ ACCETTATA" if accepted else "❌ RIFIUTATA"
        self.logger.info(f"{status} Proposta: {subject} {relation} {obj} (conf: {confidence:.2f})")
        
    def log_error(self, error: str, context: str = ""):
        """Logga un errore."""
        self.stats['errors'].append({
            'timestamp': datetime.now().isoformat(),
            'error': error,
            'context': context
        })
        self.logger.error(f"[ERRORE] {context}: {error}")
        
    def log_iteration_summary(self, iteration: int, actions: list, results: dict):
        """Logga riepilogo iterazione."""
        self.logger.info(f"\n{'='*60}")
        self.logger.info(f"ITERAZIONE {iteration} - Riepilogo")
        self.logger.info(f"Azioni: {', '.join(actions)}")
        self.logger.info(f"Risultati: {json.dumps(results, indent=2)}")
        self.logger.info(f"{'='*60}\n")
        
    def save_session(self):
        """Salva statistiche finali sessione."""
        self.stats['ended_at'] = datetime.now().isoformat()
        self.stats['duration_seconds'] = (
            datetime.fromisoformat(self.stats['ended_at']) - 
            datetime.fromisoformat(self.stats['started_at'])
        ).total_seconds()
        
        with open(self.session_file, 'w', encoding='utf-8') as f:
            json.dump(self.stats, f, indent=2, ensure_ascii=False)
            
        self.logger.info(f"\n📊 SESSIONE SALVATA in {self.session_file}")
        self.logger.info(f"   Opinioni totali: {self.stats['total_opinions']}")
        self.logger.info(f"   Proposte: {self.stats['proposals_accepted']}/{self.stats['proposals_made']}")
        
    def get_recent_opinions(self, n: int = 10, category: Optional[str] = None) -> list[Opinion]:
        """Recupera opinioni recenti dal file."""
        opinions = []
        if not self.opinion_file.exists():
            return opinions
            
        with open(self.opinion_file, 'r', encoding='utf-8') as f:
            lines = f.readlines()
            
        for line in lines[-n:]:
            try:
                data = json.loads(line)
                if category and data.get('category') != category:
                    continue
                opinions.append(Opinion(**data))
            except:
                continue
                
        return opinions


class OpinionSummarizer:
    """Crea riepiloghi delle opinioni per il context window."""
    
    def __init__(self, logger: OpinionLogger):
        self.logger = logger
        
    def summarize_for_context(self, max_chars: int = 2000) -> str:
        """
        Crea un riepilogo compatto delle opinioni recenti per inserirlo
        nel context window del modello.
        """
        opinions = self.logger.get_recent_opinions(n=20)
        
        if not opinions:
            return "Nessuna opinione precedente."
            
        # Raggruppa per categoria
        by_category = {}
        for op in opinions:
            by_category.setdefault(op.category, []).append(op)
            
        lines = ["## Opinioni recenti dell'agente:\n"]
        
        for cat in ['insight', 'concern', 'suggestion', 'observation']:
            if cat in by_category:
                lines.append(f"\n**{cat.upper()}:**")
                for op in by_category[cat][-3:]:  # ultime 3 per categoria
                    lines.append(f"  • {op.content[:100]}")
                    
        result = '\n'.join(lines)
        return result[:max_chars]
