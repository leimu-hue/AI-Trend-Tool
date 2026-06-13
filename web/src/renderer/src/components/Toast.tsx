import { createContext, useContext, useState, useCallback, useEffect, useRef, type ReactNode } from 'react'

type ToastType = 'success' | 'error' | 'info'

interface Toast {
  id: number
  type: ToastType
  message: string
  exiting: boolean
}

interface ToastContextValue {
  success: (message: string) => void
  error: (message: string) => void
  info: (message: string) => void
}

const ToastContext = createContext<ToastContextValue | null>(null)

const TIMINGS: Record<ToastType, number> = {
  success: 3000,
  error: 3000,
  info: 2000
}

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([])
  const nextId = useRef(0)
  const timersRef = useRef<Set<number>>(new Set())

  // Cleanup all active timers on unmount
  useEffect(() => {
    return () => {
      timersRef.current.forEach((id) => clearTimeout(id))
      timersRef.current.clear()
    }
  }, [])

  const remove = useCallback((id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id))
  }, [])

  const show = useCallback(
    (type: ToastType, message: string) => {
      const id = nextId.current++
      setToasts((prev) => [...prev, { id, type, message, exiting: false }])

      const duration = TIMINGS[type]
      const timer1 = window.setTimeout(() => {
        timersRef.current.delete(timer1)
        setToasts((prev) => prev.map((t) => (t.id === id ? { ...t, exiting: true } : t)))
        const timer2 = window.setTimeout(() => {
          timersRef.current.delete(timer2)
          remove(id)
        }, 300)
        timersRef.current.add(timer2)
      }, duration)
      timersRef.current.add(timer1)
    },
    [remove]
  )

  const success = useCallback((message: string) => show('success', message), [show])
  const error = useCallback((message: string) => show('error', message), [show])
  const info = useCallback((message: string) => show('info', message), [show])

  return (
    <ToastContext.Provider value={{ success, error, info }}>
      {children}
      <div className="toast-container">
        {toasts.map((t) => (
          <div
            key={t.id}
            className={`toast toast-${t.type}${t.exiting ? ' toast-exit' : ''}`}
          >
            {t.message}
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  )
}

export function useToast(): ToastContextValue {
  const ctx = useContext(ToastContext)
  if (!ctx) {
    return {
      success: () => {},
      error: () => {},
      info: () => {}
    }
  }
  return ctx
}
