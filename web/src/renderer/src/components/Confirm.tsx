import type { ReactNode } from 'react'

interface ConfirmProps {
  open: boolean
  title?: string
  message: ReactNode
  confirmText?: string
  cancelText?: string
  danger?: boolean
  onConfirm: () => void
  onCancel: () => void
}

export default function Confirm({
  open,
  title = '确认操作',
  message,
  confirmText = '确认',
  cancelText = '取消',
  danger = false,
  onConfirm,
  onCancel
}: ConfirmProps) {
  function handleOverlayClick(e: React.MouseEvent) {
    if (e.target === e.currentTarget) onCancel()
  }

  if (!open) return null

  return (
    <div className="modal-overlay open" onClick={handleOverlayClick}>
      <div className="modal" style={{ maxWidth: 400 }}>
        <h2>{title}</h2>
        <p style={{ color: 'var(--color-fg-2)', fontSize: 13.5, lineHeight: 1.6, margin: '0 0 8px' }}>
          {message}
        </p>
        <div className="modal-actions">
          <button className="btn btn-ghost btn-sm" onClick={onCancel}>
            {cancelText}
          </button>
          <button
            className={`btn btn-sm ${danger ? 'btn-danger' : 'btn-primary'}`}
            onClick={onConfirm}
          >
            {confirmText}
          </button>
        </div>
      </div>
    </div>
  )
}
