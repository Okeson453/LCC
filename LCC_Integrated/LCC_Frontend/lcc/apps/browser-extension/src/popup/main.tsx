/**
 * Popup main entry — mounts the React app into the popup index.html.
 */
import { createRoot } from 'react-dom/client';
import { App } from './App';
import '@lcc/tokens/css';
import './popup.css';

const root = document.getElementById('root');
if (root) createRoot(root).render(<App />);
