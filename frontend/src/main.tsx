import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import '@fontsource-variable/inter';
import App from './App';
import { ThemeProvider } from './theme';
import './styles.css';

const rootEl = document.getElementById('root');
if (!rootEl) {
    throw new Error('Root element #root not found');
}

createRoot(rootEl).render(
    <StrictMode>
        <ThemeProvider>
            <App />
        </ThemeProvider>
    </StrictMode>,
);
