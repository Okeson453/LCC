import { createRoot } from 'react-dom/client';
import { App } from './App';
import '@lcc/tokens/css';
import './sidepanel.css';

const root = document.getElementById('root');
if (root) createRoot(root).render(<App />);
