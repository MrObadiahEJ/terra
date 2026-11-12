import { useEffect, useRef, type PointerEvent as ReactPointerEvent, type ReactNode } from 'react'

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
  const sheetRef = useRef<HTMLElement>(null)
  const dragRef = useRef<{ startY: number; dy: number } | null>(null)

  // Swipe the bottom-sheet handle down to dismiss it (immersive atlas only;
  // the handle is hidden above 860px). Vertical drags shorter than the
  // threshold spring back, and keyboard/close-button paths are unchanged.
  const onGrabPointerDown = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (!window.matchMedia('(max-width: 860px)').matches) return
    dragRef.current = { startY: event.clientY, dy: 0 }
    if (sheetRef.current) sheetRef.current.style.transition = 'none'
    event.currentTarget.setPointerCapture(event.pointerId)
  }
  const onGrabPointerMove = (event: ReactPointerEvent<HTMLDivElement>) => {
    const drag = dragRef.current
    if (!drag) return
    drag.dy = Math.max(0, event.clientY - drag.startY)
    if (sheetRef.current) sheetRef.current.style.transform = `translateY(${drag.dy}px)`
  }
  const onGrabPointerEnd = () => {
    const drag = dragRef.current
    dragRef.current = null
    const sheet = sheetRef.current
    if (!drag || !sheet) return
    sheet.style.transform = ''
    sheet.style.transition = ''
    if (drag.dy > 90) onClose()
  }

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
        ref={sheetRef}
        className={`sidenav sidenav--${mode}${open ? ' sidenav--open' : ''}${
          className ? ` ${className}` : ''
        }`}
        aria-label={label}
        aria-hidden={!open}
        {...(mode === 'over' ? { role: 'dialog' as const, 'aria-modal': open } : {})}
      >
        <div
          className="sidenav-grab"
          aria-hidden="true"
          onPointerDown={onGrabPointerDown}
          onPointerMove={onGrabPointerMove}
          onPointerUp={onGrabPointerEnd}
          onPointerCancel={onGrabPointerEnd}
        />
        <div className="sidenav-inner">{children}</div>
      </aside>
    </>
  )
}
