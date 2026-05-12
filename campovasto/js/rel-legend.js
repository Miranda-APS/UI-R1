// Legenda interattiva dei tipi di relazione (sezione #relazioni in sidebar).
// Ogni famiglia (S/C/M/F/L) è un toggle. La freccia ► espande i sotto-tipi
// (IS_A, PART_OF, …) ciascuno con toggle individuale. I toggle pilotano
// `ui-state.js _enabledRelTypes`; quando lo stato cambia, gli archi di tipi
// disabilitati non vengono mostrati al hover/select.

import { REL_GROUPS_INFO, TYPES_BY_GROUP, REL_LEGEND_LABEL_IT, REL_GROUP } from './constants.js';
import {
  isRelTypeEnabled, getEnabledRelTypes, toggleRelType, toggleRelGroup,
} from './ui-state.js';
import { get as getField } from './manager.js';
import { applyHighlight, clearHighlight } from './graph.js';
import { getFilterDirection } from './ui-state.js';

// Tutti i tipi noti (chiavi di REL_GROUP) — usati per inizializzare il Set
// di toggle al primo click.
const ALL_TYPES = Object.keys(REL_GROUP);

// Costruisce la UI nella sezione #relLegend e cabla i listener.
// Idempotente: se chiamata di nuovo, resetta la legenda (raro).
export function buildRelLegend(){
  const host = document.getElementById('relLegend');
  if(!host) return;
  host.innerHTML = '';

  for(const info of REL_GROUPS_INFO){
    const types = TYPES_BY_GROUP[info.id] || [];

    const row = document.createElement('div');
    row.className = 'rel-row';
    row.dataset.group = info.id;
    row.innerHTML = `
      <button class="rel-toggle" data-action="toggle-group" title="attiva/disattiva ${info.label}">
        <span class="legend-line ${info.cssClass}"></span>
        <span class="rel-label">${info.label}</span>
      </button>
      <button class="rel-expand" data-action="expand" title="mostra/nascondi i tipi">►</button>
    `;
    host.appendChild(row);

    const subs = document.createElement('div');
    subs.className = 'rel-subtypes';
    subs.dataset.group = info.id;
    for(const rel of types){
      const sub = document.createElement('button');
      sub.className = 'rel-subtype';
      sub.dataset.type = rel;
      const ita = REL_LEGEND_LABEL_IT[rel] || rel.toLowerCase().replace(/_/g, ' ');
      sub.textContent = ita;
      sub.title = `attiva/disattiva «${ita}»`;
      subs.appendChild(sub);
    }
    host.appendChild(subs);
  }

  // Stato iniziale: tutti attivi.
  refreshLegendState();

  // Wiring delegato (un solo listener per host).
  host.addEventListener('click', (e) => {
    const expandBtn = e.target.closest('.rel-expand');
    if(expandBtn){
      const groupId = expandBtn.parentElement.dataset.group;
      const subs = host.querySelector(`.rel-subtypes[data-group="${groupId}"]`);
      const open = subs.classList.toggle('aperto');
      expandBtn.textContent = open ? '▼' : '►';
      return;
    }
    const groupBtn = e.target.closest('button.rel-toggle');
    if(groupBtn){
      const groupId = groupBtn.parentElement.dataset.group;
      toggleRelGroup(TYPES_BY_GROUP[groupId] || [], ALL_TYPES);
      refreshLegendState();
      reapplyToHighlight();
      return;
    }
    const subBtn = e.target.closest('button.rel-subtype');
    if(subBtn){
      const rel = subBtn.dataset.type;
      toggleRelType(rel, ALL_TYPES);
      refreshLegendState();
      reapplyToHighlight();
      return;
    }
  });
}

// Applica lo stato corrente di _enabledRelTypes alla UI: classe `attivo`
// sui toggle attivi, opacità ridotta su quelli disabilitati.
function refreshLegendState(){
  const host = document.getElementById('relLegend');
  if(!host) return;
  for(const info of REL_GROUPS_INFO){
    const types = TYPES_BY_GROUP[info.id] || [];
    const allOn  = types.every(t => isRelTypeEnabled(t));
    const someOn = types.some(t => isRelTypeEnabled(t));
    const groupBtn = host.querySelector(`.rel-row[data-group="${info.id}"] .rel-toggle`);
    if(groupBtn){
      groupBtn.classList.toggle('attivo', allOn);
      groupBtn.classList.toggle('parziale', !allOn && someOn);
    }
    const subs = host.querySelectorAll(`.rel-subtypes[data-group="${info.id}"] .rel-subtype`);
    subs.forEach(sub => {
      sub.classList.toggle('attivo', isRelTypeEnabled(sub.dataset.type));
    });
  }
}

// Se c'è una parola selezionata in un campo, ricalcola l'highlight dopo il
// cambio dei filtri di tipo: gli archi non più consentiti spariscono, quelli
// riabilitati ricompaiono.
function reapplyToHighlight(){
  const F = getField('vasto');
  const N = getField('nuovo');
  for(const F2 of [F, N]){
    if(F2 && F2.selected){
      const direction = F2.id === 'vasto' ? getFilterDirection() : 'both';
      applyHighlight(F2, F2.selected, F2.getRosa(F2.selected, direction, { filterByType: isRelTypeEnabled }));
    }
  }
}
