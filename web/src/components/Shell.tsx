import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'

type Mode = 'creator' | 'developer'

async function fetchHealth() {
  const res = await fetch('/health')
  if (!res.ok) throw new Error('health check failed')
  return res.json() as Promise<{ status: string; version: string }>
}

export function Shell() {
  const [mode, setMode] = useState<Mode>('creator')

  const { data: health } = useQuery({
    queryKey: ['health'],
    queryFn: fetchHealth,
    retry: false,
  })

  return (
    <div className="flex flex-col min-h-screen bg-gray-950 text-gray-100 font-sans">
      <Header mode={mode} onModeChange={setMode} health={health} />
      <main className="flex-1 p-6">
        {mode === 'creator' ? <CreatorView /> : <DeveloperView />}
      </main>
    </div>
  )
}

function Header({
  mode,
  onModeChange,
  health,
}: {
  mode: Mode
  onModeChange: (m: Mode) => void
  health?: { status: string; version: string }
}) {
  return (
    <header className="flex items-center justify-between px-6 py-3 border-b border-gray-800 bg-gray-900">
      <div className="flex items-center gap-3">
        <span className="text-lg font-semibold tracking-tight text-white">Duskwell</span>
        {health && (
          <span className="text-xs text-gray-500">v{health.version}</span>
        )}
      </div>

      <div className="flex items-center gap-1 bg-gray-800 rounded-lg p-1">
        <ModeButton active={mode === 'creator'} onClick={() => onModeChange('creator')}>
          Creator
        </ModeButton>
        <ModeButton active={mode === 'developer'} onClick={() => onModeChange('developer')}>
          Developer
        </ModeButton>
      </div>
    </header>
  )
}

function ModeButton({
  active,
  onClick,
  children,
}: {
  active: boolean
  onClick: () => void
  children: React.ReactNode
}) {
  return (
    <button
      onClick={onClick}
      className={[
        'px-4 py-1.5 rounded-md text-sm font-medium transition-colors',
        active
          ? 'bg-indigo-600 text-white shadow'
          : 'text-gray-400 hover:text-gray-200',
      ].join(' ')}
    >
      {children}
    </button>
  )
}

function CreatorView() {
  return (
    <div className="flex flex-col items-center justify-center h-64 gap-3 text-gray-500">
      <div className="text-4xl">🖼</div>
      <p className="text-sm">Creator view — asset gallery coming in Phase 2</p>
    </div>
  )
}

function DeveloperView() {
  return (
    <div className="flex flex-col items-center justify-center h-64 gap-3 text-gray-500">
      <div className="text-4xl">🌲</div>
      <p className="text-sm">Developer view — file tree coming in Phase 1</p>
    </div>
  )
}
