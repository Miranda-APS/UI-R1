// Sidebar collapsible + drag-to-resize. Persiste in localStorage.

import { LS } from '../constants.js';
import { network } from '../graph.js';

export function wireSidebarLayout(){
  const sb = document.getElementById('sidebar');
  const handle = document.getElementById('sidebarResizer');
  const toggle = document.getElementById('sidebarToggle');
  if(!sb) return;

  const setCollapsed = (val) => {
    sb.classList.toggle('collapsed', val);
    document.body.classList.toggle('sidebar-collapsed', val);
    if(toggle) toggle.textContent = val ? '›' : '‹';
  };
  const setWidthVar = (w) => {
    document.documentElement.style.setProperty('--sidebar-w', w + 'px');
    sb.style.setProperty('--sidebar-w', w + 'px');
  };

  const savedW = parseInt(localStorage.getItem(LS.SIDEBAR_W) || '510');
  if(savedW >= 220 && savedW <= window.innerWidth * 0.8) setWidthVar(savedW);
  else setWidthVar(510);
  const savedCollapsed = localStorage.getItem(LS.SIDEBAR_COLLAPSED) === '1';
  setCollapsed(savedCollapsed);

  if(handle){
    let dragging = false, startX = 0, startW = 510;
    handle.addEventListener('mousedown', e => {
      if(sb.classList.contains('collapsed')) return;
      dragging = true;
      startX = e.clientX;
      startW = sb.getBoundingClientRect().width;
      handle.classList.add('dragging');
      document.body.style.userSelect = 'none';
      e.preventDefault();
    });
    document.addEventListener('mousemove', e => {
      if(!dragging) return;
      const dx = startX - e.clientX;
      const newW = Math.max(220, Math.min(window.innerWidth * 0.8, startW + dx));
      setWidthVar(newW);
    });
    document.addEventListener('mouseup', () => {
      if(!dragging) return;
      dragging = false;
      handle.classList.remove('dragging');
      document.body.style.userSelect = '';
      const w = sb.getBoundingClientRect().width;
      try { localStorage.setItem(LS.SIDEBAR_W, String(Math.round(w))); } catch(_){}
      try { network?.redraw(); network?.fit({ animation: false }); } catch(_){}
    });
  }

  if(toggle){
    toggle.onclick = () => {
      const isCollapsed = !sb.classList.contains('collapsed');
      setCollapsed(isCollapsed);
      try { localStorage.setItem(LS.SIDEBAR_COLLAPSED, isCollapsed ? '1' : '0'); } catch(_){}
      try { network?.redraw(); network?.fit({ animation: true }); } catch(_){}
    };
  }
}
