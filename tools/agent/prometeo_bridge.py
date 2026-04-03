"""
Bridge tra l'agente Python e il sistema Prometeo (Rust).
Legge/scrive su KG JSON e topology state binario.
"""
import json
import struct
from pathlib import Path
from typing import Optional
from dataclasses import dataclass, asdict
from collections import defaultdict


@dataclass
class TopologyStats:
    """Statistiche sullo stato del campo."""
    word_count: int
    edge_count: int
    kg_nodes: int
    kg_edges: int
    isolated_words: list[str]
    densest_nodes: list[tuple[str, int]]
    
    def to_dict(self):
        return asdict(self)


@dataclass
class KgEdge:
    subject: str
    relation: str
    object: str
    confidence: float
    source: str


class PrometeoBridge:
    """
    Bridge verso il sistema Prometeo.
    Legge il KG da JSON e lo stato topologico dal file binario.
    """
    
    def __init__(
        self,
        kg_path: str = None,
        state_path: str = None,
    ):
        # Default: cerca nella directory parent (project root)
        root_dir = Path(__file__).parent.parent.parent
        self.kg_path = Path(kg_path) if kg_path else root_dir / "prometeo_kg.json"
        self.state_path = Path(state_path) if state_path else root_dir / "prometeo_topology_state.bin"
        
        # Cache in memoria
        self._kg_edges: list[KgEdge] = []
        self._kg_index: dict[str, list[KgEdge]] = defaultdict(list)
        self._word_list: list[str] = []
        self._topology_edges: list[tuple[str, str, float]] = []  # (w1, w2, weight)
        
        self._load_kg()
        self._load_topology_state()
        
    def _load_kg(self):
        """Carica il Knowledge Graph da JSON."""
        if not self.kg_path.exists():
            raise FileNotFoundError(f"KG non trovato: {self.kg_path}")
            
        with open(self.kg_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
            
        self._kg_edges = []
        self._kg_index = defaultdict(list)
        
        for edge_data in data.get('edges', []):
            edge = KgEdge(
                subject=edge_data['subject'],
                relation=edge_data['relation'],
                object=edge_data['object'],
                confidence=edge_data.get('confidence', 1.0),
                source=edge_data.get('source', 'Unknown')
            )
            self._kg_edges.append(edge)
            self._kg_index[edge.subject].append(edge)
            self._kg_index[edge.object].append(edge)
            
    def _load_topology_state(self):
        """
        Legge lo stato binario di Prometeo.
        Formato: [header] + [word_list] + [edges]
        """
        if not self.state_path.exists():
            raise FileNotFoundError(f"Stato non trovato: {self.state_path}")
            
        with open(self.state_path, 'rb') as f:
            # Header: magic (4) + version (4) + word_count (4)
            magic = f.read(4)
            if magic != b'PRMT':
                print(f"[WARN] Magic bytes non corrispondono: {magic}")
                
            version = struct.unpack('I', f.read(4))[0]
            word_count = struct.unpack('I', f.read(4))[0]
            
            # Leggi word list
            self._word_list = []
            for _ in range(min(word_count, 30000)):  # Safety limit
                try:
                    len_byte = f.read(1)
                    if not len_byte:
                        break
                    word_len = len_byte[0]
                    word = f.read(word_len).decode('utf-8', errors='ignore')
                    self._word_list.append(word)
                except:
                    break
                    
        print(f"[Bridge] Caricate {len(self._word_list)} parole, {len(self._kg_edges)} archi KG")
    
    # ═══════════════════════════════════════════════════════════════════════
    # API per l'agente
    # ═══════════════════════════════════════════════════════════════════════
    
    def get_stats(self) -> TopologyStats:
        """Restituisce statistiche sul campo."""
        # Trova parole isolate (nel lessico ma non nel KG)
        kg_words = set()
        for edge in self._kg_edges:
            kg_words.add(edge.subject)
            kg_words.add(edge.object)
            
        isolated = [w for w in self._word_list if w not in kg_words][:50]
        
        # Nodi più densi (più connessioni)
        connection_count = defaultdict(int)
        for edge in self._kg_edges:
            connection_count[edge.subject] += 1
            connection_count[edge.object] += 1
            
        densest = sorted(
            connection_count.items(),
            key=lambda x: x[1],
            reverse=True
        )[:20]
        
        return TopologyStats(
            word_count=len(self._word_list),
            edge_count=len(self._topology_edges),
            kg_nodes=len(kg_words),
            kg_edges=len(self._kg_edges),
            isolated_words=isolated,
            densest_nodes=densest
        )
    
    def query_concept(self, concept: str, relation_type: str = "Any") -> dict:
        """
        Interroga il KG per un concetto.
        
        Returns:
            {
                'concept': str,
                'exists': bool,
                'outgoing': list[edge],
                'incoming': list[edge],
                'similar': list[str],
                'opposite': list[str],
                'is_a': list[str]
            }
        """
        concept = concept.lower().strip()
        
        outgoing = []
        incoming = []
        similar = []
        opposite = []
        is_a = []
        
        for edge in self._kg_edges:
            # Outgoing
            if edge.subject == concept:
                if relation_type == "Any" or edge.relation == relation_type:
                    outgoing.append({
                        'relation': edge.relation,
                        'object': edge.object,
                        'confidence': edge.confidence
                    })
                if edge.relation == 'SimilarTo':
                    similar.append(edge.object)
                elif edge.relation == 'OppositeOf':
                    opposite.append(edge.object)
                elif edge.relation == 'IsA':
                    is_a.append(edge.object)
                    
            # Incoming
            if edge.object == concept:
                if relation_type == "Any" or edge.relation == relation_type:
                    incoming.append({
                        'relation': edge.relation,
                        'subject': edge.subject,
                        'confidence': edge.confidence
                    })
        
        return {
            'concept': concept,
            'exists': concept in self._word_list or len(outgoing) > 0 or len(incoming) > 0,
            'outgoing': outgoing,
            'incoming': incoming,
            'similar': similar,
            'opposite': opposite,
            'is_a': is_a
        }
    
    def find_isolated_nodes(self, min_connections: int = 2) -> list[str]:
        """Trova nodi con meno di N connessioni."""
        connection_count = defaultdict(int)
        for edge in self._kg_edges:
            connection_count[edge.subject] += 1
            connection_count[edge.object] += 1
            
        return [word for word in self._word_list 
                if connection_count.get(word, 0) < min_connections][:100]
    
    def find_semantic_gaps(self) -> list[dict]:
        """
        Trova "buchi" semantici: coppie di concetti simili che non sono connessi
        ma dovrebbero esserlo.
        """
        gaps = []
        
        # Per ogni concetto, guarda i suoi similar
        for edge in self._kg_edges:
            if edge.relation == 'SimilarTo':
                # Trova similar del similar
                similar_of_similar = self._get_similar(edge.object)
                for sim in similar_of_similar:
                    if sim != edge.subject and sim not in self._get_similar(edge.subject):
                        # Potenziale gap: A~B, B~C, ma A non ~C
                        gaps.append({
                            'concept_a': edge.subject,
                            'concept_b': edge.object,
                            'concept_c': sim,
                            'gap_type': 'transitive_similarity'
                        })
                        
        return gaps[:20]
    
    def _get_similar(self, concept: str) -> set[str]:
        """Utility: restituisce tutti i SimilarTo di un concetto."""
        result = set()
        for edge in self._kg_edges:
            if edge.relation == 'SimilarTo':
                if edge.subject == concept:
                    result.add(edge.object)
                elif edge.object == concept:
                    result.add(edge.subject)
        return result
    
    def check_fractal_consistency(self, concept: str) -> dict:
        """
        Verifica coerenza frattale (stub - richiede integrazione con frattali).
        Per ora restituisce analisi basata sulle relazioni.
        """
        info = self.query_concept(concept)
        
        # Euristiche semplificate per coerenza
        warnings = []
        
        # Se ha molti OppositeOf, probabilmente è un concetto con polarità forte
        if len(info['opposite']) > 3:
            warnings.append("Troppi opposti - possibile sovraccarico semantico")
            
        # Se non ha SimilarTo ma ha IsA, potrebbe essere troppo astratto
        if not info['similar'] and len(info['is_a']) > 2:
            warnings.append("Nodo astratto senza similitudini concrete")
            
        return {
            'concept': concept,
            'warnings': warnings,
            'coherence_score': 1.0 - (len(warnings) * 0.2),
            'suggested_fractal': None  # TODO: mappatura concetto→frattale
        }
    
    def add_proposed_edge(self, subject: str, relation: str, obj: str, 
                          confidence: float, reasoning: str) -> bool:
        """
        Aggiunge un arco proposto al KG (in memoria, non salva ancora).
        Returns True se aggiunto, False se duplicato.
        """
        # Check duplicato
        for edge in self._kg_edges:
            if (edge.subject == subject and 
                edge.relation == relation and 
                edge.object == obj):
                return False
                
        new_edge = KgEdge(
            subject=subject,
            relation=relation,
            object=obj,
            confidence=confidence,
            source='AgentProposed'
        )
        self._kg_edges.append(new_edge)
        self._kg_index[subject].append(new_edge)
        self._kg_index[obj].append(new_edge)
        return True
    
    def save_kg(self, output_path: Optional[str] = None):
        """Salva il KG modificato su disco."""
        path = Path(output_path) if output_path else self.kg_path
        
        data = {
            'edges': [
                {
                    'subject': e.subject,
                    'relation': e.relation,
                    'object': e.object,
                    'confidence': e.confidence,
                    'source': e.source
                }
                for e in self._kg_edges
            ]
        }
        
        with open(path, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
            
        print(f"[Bridge] KG salvato: {path} ({len(self._kg_edges)} archi)")
