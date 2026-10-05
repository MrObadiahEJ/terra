import { useEffect, type ReactNode } from 'react'

export type SidenavMode = 'side' | 'over'

interface SidenavProps {
  /** Open state owned by the parent; it intentionally does not persist across visits. */
  open: boolean
  /** side = docked in the layout flow; over = floats above the content behind a scrim. */
  mode: SidenavMode
  /** Close requests: scrim click or Escape. The parent's own buttons call it too. */
  onClose: () => void
  /** Accessible name of the drawer. */
  label: string
  /** Extra class name applied to the <aside> for page-specific layout. */
  className?: string
  children: ReactNode
}

export default function Sidenav({ open, mode, onClose, label, className, children }: SidenavProps) {
  useEffect(() => {
    if (!open) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [open, onClose])

  return (
    <>
      <div
        className={`sidenav-scrim sidenav-scrim--${mode}${open ? ' sidenav-scrim--visible' : ''}`}
        onClick={onClose}
        aria-hidden="true"
      />
      <aside
        className={`sidenav sidenav--${mode}${open ? ' sidenav--open' : ''}${
          className ? ` ${className}` : ''
        }`}
        aria-label={label}
        aria-hidden={!open}
        {...(mode === 'over' ? { role: 'dialog' as const, 'aria-modal': open } : {})}
      >
        <div className="sidenav-inner">{children}</div>
      </aside>
    </>
  )
}
