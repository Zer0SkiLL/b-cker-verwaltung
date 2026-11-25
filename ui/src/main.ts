import { mount } from 'svelte';
import App from './App.svelte';
import './app.css';
import { recordError } from './lib/diagnostics';

// Unerwartete Fehler für „Problem melden“ festhalten (nur Fehlertext, keine Daten).
window.addEventListener('error', (e) => recordError(e.error ?? e.message));
window.addEventListener('unhandledrejection', (e) => recordError(e.reason));

const app = mount(App, { target: document.getElementById('app')! });

export default app;
