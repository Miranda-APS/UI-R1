// Modale di conferma interna — sostituisce confirm()/alert() nativi.
// Riusa l'infrastruttura di edit-panel.js (backdrop, ESC, stile coerente).
//
// API:
//   await openConfirm({
//     title: 'Svuotare il campo nuovo?',
//     message: 'Tutte le parole e relazioni andranno perse.',
//     confirmLabel: 'svuota',
//     cancelLabel: 'annulla',
//     danger: true,        // colora il bottone primario di rosso
//   });  // → boolean
//
// Per scelte multiple (3+ opzioni), vedi openChoice più sotto.

import { openPanel, closePanel } from './edit-panel.js';

// Conferma binaria sì/no. Ritorna Promise<boolean>.
// Risolve true se l'utente clicca confirm, false se annulla / ESC / backdrop.
export function openConfirm({
  title = '',
  message = '',
  confirmLabel = 'conferma',
  cancelLabel = 'annulla',
  danger = false,
} = {}){
  return new Promise((resolve) => {
    let resolved = false;
    const finish = (val) => {
      if(resolved) return;
      resolved = true;
      resolve(val);
    };
    openPanel({
      title,
      build: (body) => {
        if(message){
          const p = document.createElement('p');
          p.className = 'confirm-message';
          // message può essere multiline: \n → <br>. Niente innerHTML
          // diretto per evitare XSS via testo di parole utente.
          message.split('\n').forEach((line, i) => {
            if(i > 0) p.appendChild(document.createElement('br'));
            p.appendChild(document.createTextNode(line));
          });
          body.appendChild(p);
        }
      },
      actions: [
        {
          label: confirmLabel,
          primary: !danger,
          danger: !!danger,
          onClick: () => { finish(true); closePanel(); },
        },
        {
          label: cancelLabel,
          onClick: () => { finish(false); closePanel(); },
        },
      ],
      onClose: () => finish(false),  // chiusura via backdrop / ESC
    });
  });
}

// Scelta fra N opzioni. Ritorna Promise<key|null>: la key dell'azione
// cliccata, o null se annulla / ESC / backdrop.
//
// Esempio (svuota campo con lavoro pendente):
//   const choice = await openChoice({
//     title: 'Hai N elementi non trasmessi',
//     message: 'Cosa vuoi fare prima di svuotare?',
//     choices: [
//       { key: 'transmit', label: 'trasmetti e svuota', primary: true },
//       { key: 'discard',  label: 'svuota senza trasmettere', danger: true },
//       { key: 'cancel',   label: 'annulla' },
//     ],
//   });
//   if(choice === 'transmit'){ ... }
export function openChoice({
  title = '',
  message = '',
  choices = [],
} = {}){
  return new Promise((resolve) => {
    let resolved = false;
    const finish = (val) => {
      if(resolved) return;
      resolved = true;
      resolve(val);
    };
    openPanel({
      title,
      build: (body) => {
        if(message){
          const p = document.createElement('p');
          p.className = 'confirm-message';
          message.split('\n').forEach((line, i) => {
            if(i > 0) p.appendChild(document.createElement('br'));
            p.appendChild(document.createTextNode(line));
          });
          body.appendChild(p);
        }
      },
      actions: choices.map(c => ({
        label: c.label,
        primary: !!c.primary,
        danger: !!c.danger,
        onClick: () => { finish(c.key); closePanel(); },
      })),
      onClose: () => finish(null),
    });
  });
}
