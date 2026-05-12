// Regole "chi può, come, con quali flag" per le mutazioni sui Field.
// Fonte unica di verità: nessuna parte di campovasto/ deve scrivere
// `userCreated:`, `transmitted: false` a mano.
//
// Layer logico tra Field (struttura dati) e UI (editor.js, sentence.js,
// sidebar.js). Cambiare una regola qui propaga ovunque.
//
// Tre famiglie di regole:
//   1. PERMESSI:   isReadonly(F)
//   2. FLAG NUOVI: flagsForNewWord(F, opts), flagsForNewEdge(F)
//   3. FLAG EDIT:  applyEditFlagsToWord(w), applyEditFlagsToEdge(e)
//
// Edit dimensioni in vasto: ammesso. Le modifiche sono effimere (vasto non
// persiste in localStorage — saveField('vasto') è no-op in manager.js).
// Servono per visualizzare in tempo reale la posizione corrispondente alla
// firma 8D. La "voce stabile" del vasto resta quella server, ricaricata da API.

// Read-only check — usato dalla UI per disabilitare drag/edit.
export function isReadonly(F){
  return !F;
}

// Flag preset per una NUOVA parola creata nel field F.
// opts: { fromSentence, fromApi, unknown, transmitted }
//   - fromSentence: la parola viene da una frase (alone dorato)
//   - fromApi: signature ricevuta da /api/medio (non da vasto)
//   - unknown: la parola non era nel KG e non ha signature derivata
//   - transmitted: già trasmessa (true per parole hydrate dal personale salvato)
export function flagsForNewWord(F, opts = {}){
  const isUser = F && F.id !== 'vasto';
  return {
    userCreated: isUser,
    transmitted: !!opts.transmitted,
    fromSentence: !!opts.fromSentence,
    fromApi:      !!opts.fromApi,
    noSignature:  !!opts.noSignature,
    unknown:      !!opts.unknown,
  };
}

// Flag preset per un NUOVO arco creato nel field F.
export function flagsForNewEdge(F, opts = {}){
  const isUser = F && F.id !== 'vasto';
  return {
    userCreated: isUser,
    transmitted: !!opts.transmitted,
  };
}

// Effetti collaterali quando l'utente MODIFICA la firma di una parola.
// La parola viene marcata "dirty" (da ritrasmettere) e perde lo stato
// "unknown" — se l'utente sta dando una firma, la parola non è più ignota.
// Da chiamare DOPO Field.updateWordSig().
export function applyEditFlagsToWord(w){
  if(!w?.flags) return;
  w.flags.transmitted = false;
  w.flags.userCreated = true;
  if(w.flags.unknown) w.flags.unknown = false;
}

// Effetti collaterali quando un arco viene modificato (tipo o forza).
export function applyEditFlagsToEdge(e){
  if(!e?.flags) return;
  e.flags.transmitted = false;
}
