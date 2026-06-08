import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { HashRouter } from 'react-router-dom'
import { ThemeProvider } from './theme/config'
import ErrorBoundary from './components/ErrorBoundary'
import App from './App'
import './styles/index.css'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ThemeProvider>
      <ErrorBoundary>
        <HashRouter>
          <App />
        </HashRouter>
      </ErrorBoundary>
    </ThemeProvider>
  </StrictMode>
)
