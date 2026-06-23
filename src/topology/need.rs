//! Phase 86+ — Il BISOGNO che un input apre nel campo.
//!
//! > Design: `docs/raw/architettura/comprensione_bisogno_atto.md`.
//!
//! Comprendere ≠ ridire l'arco più forte. Un input perturba il campo e vi apre
//! un **bisogno**; l'atto (altrove) lo scioglie. Questo modulo è la sola
//! **lettura**: compone segnali GIÀ calcolati dagli organi esistenti (grafo di
//! comprensione, confronto, closure, coerenza, …) e nomina la tensione
//! dominante. Non decide nulla, non porta stato nuovo, non introduce soglie.
//!
//! ## Disciplina (Test Pre-Proposta §7 del design)
//!
//! - **Forma, non trigger**: legge *come* la comprensione ha deformato il campo,
//!   mai *quando* fare X.
//! - **Niente numeri-magici di switch**: la selezione è `argmax` di intensità
//!   continue (stessa fisica di `select_pattern_by_resonance` e del percetto
//!   `vicinanza=|CD5|` di Phase 83). NON esiste `if intensità > X → bisogno Y`.
//!   Dove più segnali alimentano un bisogno si prende il **max** dei segnali
//!   (già normalizzati in [0,1]) — niente pesi da tarare.
//! - **Spiegabile**: l'intensità di ogni bisogno È un quantità del campo.
//!
//! Modulo **additivo e puro**: non tocca `compose`. L'engine lo calcola come
//! `last_need` (osservabile), e SOLO in un secondo tempo l'atto lo consumerà.

/// La tensione che un input apre nel campo. Una sola domina per turno (argmax);
/// le altre restano sfondo (FILOSOFIA §desideri).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Need {
    /// Slot non saturo / nodo non fondato → la *sola* domanda che sblocca.
    Capire,
    /// Un gap aperto prima si chiude ora → restituire la struttura.
    Riconoscere,
    /// Relazione nuova/contraddetta dal mondo, o tocco di una pendenza del sé →
    /// prendere posizione (con un perché).
    Posizionarsi,
    /// Sovraccarico/frammentazione → calmarsi, tenere (atto = il meno).
    CoRegolare,
    /// Un fatto/intenzione del parlante riaffiora → ridarlo senza colpa.
    EsternalizzareMemoria,
    /// Assenza poi ritorno → riconoscimento relazionale, non notifica.
    RistabilireRelazione,
    /// Molte proposizioni / componenti sconnesse → ordinare, una cosa alla volta.
    Strutturare,
}

impl Need {
    pub fn as_str(&self) -> &'static str {
        match self {
            Need::Capire => "capire",
            Need::Riconoscere => "riconoscere",
            Need::Posizionarsi => "posizionarsi",
            Need::CoRegolare => "co-regolare",
            Need::EsternalizzareMemoria => "esternalizzare-memoria",
            Need::RistabilireRelazione => "ristabilire-relazione",
            Need::Strutturare => "strutturare",
        }
    }

    /// Nome del percetto omogeneo al kg_proc (per il futuro seeding nel campo
    /// procedurale, `KgProcActivation::seed_percetto`). Alcuni coincidono con i
    /// percetti già curati (apertura/chiusura/posizione); gli altri sono ganci
    /// per curation futura — l'enum NON dispatcha, è solo un'etichetta.
    pub fn percetto(&self) -> &'static str {
        match self {
            Need::Capire => "apertura",
            Need::Riconoscere => "chiusura",
            Need::Posizionarsi => "posizione",
            Need::CoRegolare => "sovraccarico",
            Need::EsternalizzareMemoria => "memoria",
            Need::RistabilireRelazione => "assenza",
            Need::Strutturare => "struttura",
        }
    }
}

/// Segnali grezzi, già calcolati dagli organi. L'engine li riempie; questo
/// modulo li legge. Ogni campo continuo è atteso in [0,1] (intensità) o un
/// conteggio (normalizzato qui dentro). Nessun campo è una soglia.
#[derive(Debug, Clone, Default)]
pub struct NeedSignals {
    /// Nodi-contenuto del grafo che non raggiungono ancora un'ancora.
    pub ungrounded_count: usize,
    /// Totale nodi-contenuto (per normalizzare l'ungrounded).
    pub content_count: usize,
    /// Esiste un vuoto dialogico PROP-driven (signifier_gap non vuoto, Phase 83)?
    pub has_dialogic_gap: bool,
    /// Un gap aperto in un turno precedente si è chiuso ORA (Phase 78)?
    pub closes_prior_gap: bool,
    /// Magnitudine del confronto col mondo: 0 = conferma/n.a., ~0.6 = novità,
    /// 1.0 = contraddizione (il mondo tiene ciò che la frase nega). È una
    /// *lettura* dell'enum `Confront`, non una soglia.
    pub world_confront: f64,
    /// Il mondo CONFERMA il claim (la triple esiste già nel kg_sem,
    /// `Confront::Confirm`) [0,1]. L'eco fondata: l'Altro dice ciò che
    /// l'entità già tiene → il bisogno è RICONOSCERE (restituire la
    /// conoscenza condivisa), non posizionarsi né capire.
    pub world_confirm: f64,
    /// Salienza della grana del sé toccata dalla frase [0,1] (0 finché la grana
    /// non è un pesatore di salienza — Anello 1).
    pub self_salience: f64,
    /// Sovraccarico [0,1]: tipicamente `1 - coherence_integrity` (eventualmente
    /// modulato dalla saturazione del campo).
    pub overload: f64,
    /// Rilevanza di un fatto del parlante che riaffiora [0,1] (0 = nessuno).
    pub memory_resurfaced: f64,
    /// Assenza percepita al ritorno [0,1] (caduta di presenza; 0 = continuità).
    pub absence: f64,
    /// Numero di loci/componenti sconnesse della comprensione (1 = monolocus).
    pub locus_count: usize,
}

/// Esito della lettura: il bisogno dominante + l'intensità + la classifica
/// completa (per ispezione/diagnostica).
#[derive(Debug, Clone)]
pub struct NeedReading {
    pub dominant: Need,
    pub intensity: f64,
    pub ranked: Vec<(Need, f64)>,
    /// I segnali grezzi che hanno prodotto questa lettura — il *perché*. Il
    /// bisogno non è mai un'etichetta opaca: porta con sé le quantità di campo
    /// che lo giustificano (gap, confronto col mondo, salienza, loci…), così
    /// l'esplorazione può risalire dalla macro-categoria alla sua causa.
    pub signals: NeedSignals,
}

/// L'intensità di ciascun bisogno come **lettura continua** dei segnali.
/// Dove più segnali alimentano un bisogno → `max` (nessun peso da tarare).
fn intensity(need: Need, s: &NeedSignals) -> f64 {
    let ungrounded_ratio = if s.content_count > 0 {
        (s.ungrounded_count as f64 / s.content_count as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let structure = if s.locus_count > 1 {
        // più loci = più bisogno di strutturare; saturazione dolce a 3 loci.
        (((s.locus_count - 1) as f64) / 2.0).min(1.0)
    } else {
        0.0
    };
    match need {
        Need::Capire => ungrounded_ratio.max(if s.has_dialogic_gap { 1.0 } else { 0.0 }),
        Need::Riconoscere => {
            let closure: f64 = if s.closes_prior_gap { 1.0 } else { 0.0 };
            closure.max(s.world_confirm.clamp(0.0, 1.0))
        }
        Need::Posizionarsi => s.world_confront.max(s.self_salience).clamp(0.0, 1.0),
        Need::CoRegolare => s.overload.clamp(0.0, 1.0),
        Need::EsternalizzareMemoria => s.memory_resurfaced.clamp(0.0, 1.0),
        Need::RistabilireRelazione => s.absence.clamp(0.0, 1.0),
        Need::Strutturare => structure,
    }
}

/// Tutti i bisogni in ordine canonico (per ranking deterministico a parità).
const ALL: [Need; 7] = [
    Need::Capire,
    Need::Riconoscere,
    Need::Posizionarsi,
    Need::CoRegolare,
    Need::EsternalizzareMemoria,
    Need::RistabilireRelazione,
    Need::Strutturare,
];

/// Legge il bisogno dominante dai segnali. `argmax` puro delle intensità
/// continue — nessuna soglia di switch. Ritorna `None` se nessun bisogno
/// emerge (tutte le intensità a 0): è onesto — non ogni input apre una tensione
/// (un puro acknowledgment non chiede nulla).
pub fn sense_need(signals: &NeedSignals) -> Option<NeedReading> {
    let mut ranked: Vec<(Need, f64)> =
        ALL.iter().map(|&n| (n, intensity(n, signals))).collect();
    // Ordine decrescente per intensità; a parità, ordine canonico (stabile).
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let (dominant, intensity) = ranked[0];
    if intensity <= 0.0 {
        return None;
    }
    Some(NeedReading { dominant, intensity, ranked, signals: signals.clone() })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> NeedSignals {
        NeedSignals { content_count: 2, ..Default::default() }
    }

    #[test]
    fn vuoto_dialogico_apre_capire() {
        // "ho paura" (via=None) → gap PROP-driven → bisogno = capire.
        let s = NeedSignals { has_dialogic_gap: true, ..base() };
        let r = sense_need(&s).unwrap();
        assert_eq!(r.dominant, Need::Capire);
    }

    #[test]
    fn closure_apre_riconoscere() {
        // "del buio" dopo "ho paura" → chiude il gap → bisogno = riconoscere.
        let s = NeedSignals { closes_prior_gap: true, ..base() };
        assert_eq!(sense_need(&s).unwrap().dominant, Need::Riconoscere);
    }

    #[test]
    fn contraddizione_apre_posizionarsi() {
        // Il mondo tiene ciò che la frase nega → confront forte → posizionarsi.
        let s = NeedSignals { world_confront: 1.0, ..base() };
        assert_eq!(sense_need(&s).unwrap().dominant, Need::Posizionarsi);
    }

    #[test]
    fn sovraccarico_apre_coregolare() {
        // coherence bassa → overload alto → co-regolare, anche con un filo di gap.
        let s = NeedSignals { overload: 0.9, ungrounded_count: 1, content_count: 4, ..base() };
        assert_eq!(sense_need(&s).unwrap().dominant, Need::CoRegolare);
    }

    #[test]
    fn dominante_e_argmax_non_soglia() {
        // Due tensioni: vince la più intensa, senza alcuna soglia.
        let s = NeedSignals { world_confront: 0.6, overload: 0.8, ..base() };
        assert_eq!(sense_need(&s).unwrap().dominant, Need::CoRegolare);
        let s2 = NeedSignals { world_confront: 0.9, overload: 0.3, ..base() };
        assert_eq!(sense_need(&s2).unwrap().dominant, Need::Posizionarsi);
    }

    #[test]
    fn nessuna_tensione_nessun_bisogno() {
        // Campo quieto, frase fondata e concorde → nessun bisogno (onesto).
        assert!(sense_need(&base()).is_none());
    }

    #[test]
    fn multi_locus_apre_strutturare() {
        let s = NeedSignals { locus_count: 3, ..base() };
        assert_eq!(sense_need(&s).unwrap().dominant, Need::Strutturare);
    }

    #[test]
    fn conferma_del_mondo_apre_riconoscere() {
        // "la paura è un'emozione" — la triple esiste già nel kg_sem → l'eco
        // fondata apre RICONOSCERE (restituire la conoscenza condivisa).
        let s = NeedSignals { world_confirm: 1.0, ..base() };
        assert_eq!(sense_need(&s).unwrap().dominant, Need::Riconoscere);
    }
}
