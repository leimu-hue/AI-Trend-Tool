import { useState, useEffect, useCallback } from 'react'
import { Outlet, useLocation, useNavigate } from 'react-router-dom'
import dayjs from 'dayjs'
import utc from 'dayjs/plugin/utc'

dayjs.extend(utc)

const pageTitles: Record<string, string> = {
  '/dashboard': '仪表盘',
  '/articles': '文章日志',
  '/sources': '数据源管理',
  '/keywords': '关键词管理',
  '/channels': '推送渠道管理',
  '/tokens': 'API 令牌管理',
  '/settings': '系统设置'
}

interface NavItem {
  key: string
  label: string
  icon: string
}

const navGroups: { section: string; items: NavItem[] }[] = [
  {
    section: '监控',
    items: [
      { key: '/dashboard', label: '仪表盘', icon: 'grid' },
      { key: '/articles', label: '文章日志', icon: 'file' }
    ]
  },
  {
    section: '配置',
    items: [
      { key: '/sources', label: '数据源', icon: 'settings' },
      { key: '/keywords', label: '关键词', icon: 'tag' },
      { key: '/channels', label: '推送渠道', icon: 'bell' },
      { key: '/tokens', label: 'API 令牌', icon: 'lock' }
    ]
  },
  {
    section: '系统',
    items: [{ key: '/settings', label: '设置', icon: 'sliders' }]
  }
]

const svgIcons: Record<string, string> = {
  grid: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>`,
  file: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>`,
  settings: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>`,
  tag: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>`,
  bell: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>`,
  lock: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>`,
  sliders: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="3"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>`
}

export default function Layout() {
  const [isMobile, setIsMobile] = useState(window.innerWidth <= 768)
  const [mobileOpen, setMobileOpen] = useState(false)
  const [time, setTime] = useState('')
  const location = useLocation()
  const navigate = useNavigate()

  const currentTitle = pageTitles[location.pathname] || ''

  useEffect(() => {
    setTime(dayjs.utc().format('YYYY-MM-DD HH:mm:ss') + ' UTC')
    const id = setInterval(() => {
      setTime(dayjs.utc().format('YYYY-MM-DD HH:mm:ss') + ' UTC')
    }, 1000)
    return () => clearInterval(id)
  }, [])

  const handleResize = useCallback(() => {
    const mobile = window.innerWidth <= 768
    setIsMobile(mobile)
    if (!mobile) {
      setMobileOpen(false)
    }
  }, [])

  useEffect(() => {
    window.addEventListener('resize', handleResize)
    return () => window.removeEventListener('resize', handleResize)
  }, [handleResize])

  function navigateTo(key: string) {
    navigate(key)
    setMobileOpen(false)
  }

  return (
    <div className="flex h-screen overflow-hidden">
      {/* Sidebar */}
      <aside
        className={[
          'w-[220px] min-w-[220px] h-screen bg-surface border-r border-border',
          'flex flex-col z-50 transition-transform duration-200 ease-[cubic-bezier(0.2,0,0,1)]',
          isMobile
            ? `fixed left-0 top-0 ${mobileOpen ? 'translate-x-0' : '-translate-x-full'}`
            : ''
        ].join(' ')}
      >
        {/* Brand */}
        <div className="px-5 pt-5 pb-4 border-b border-border">
          <div className="flex items-center gap-2.5">
            <div className="w-7 h-7 rounded-[7px] bg-accent flex items-center justify-center font-mono text-[13px] text-fg">
              ◈
            </div>
            <div>
              <div className="font-body font-normal text-[15px] text-fg tracking-[-0.01em]">
                AI 热点监控
              </div>
              <div className="font-mono text-[10px] text-muted uppercase tracking-[0.12em] mt-1">
                多源实时监控 · 关键词告警 · 趋势分析
              </div>
            </div>
          </div>
        </div>

        {/* Nav */}
        <nav className="flex-1 p-3 flex flex-col gap-0.5 overflow-y-auto">
          {navGroups.map((group) => (
            <div key={group.section}>
              <div className="text-[10px] text-meta uppercase tracking-[0.16em] px-3 pt-4 pb-1.5 font-mono">
                {group.section}
              </div>
              {group.items.map((item) => {
                const isActive = location.pathname === item.key
                return (
                  <button
                    key={item.key}
                    onClick={() => navigateTo(item.key)}
                    className={[
                      'flex items-center gap-2.5 px-3 py-2.5 rounded-sm text-[13.5px]',
                      'cursor-pointer border-none w-full text-left font-body',
                      'transition-[background,color] duration-150',
                      isActive
                        ? 'text-fg bg-accent'
                        : 'text-muted bg-transparent hover:bg-accent/50'
                    ].join(' ')}
                  >
                    <span
                      dangerouslySetInnerHTML={{ __html: svgIcons[item.icon] || '' }}
                      className={`w-4 h-4 shrink-0 ${isActive ? 'opacity-100' : 'opacity-60'}`}
                    />
                    {item.label}
                  </button>
                )
              })}
            </div>
          ))}
        </nav>

        {/* Footer */}
        <div className="px-4 py-3 border-t border-border flex flex-col gap-1">
          <div className="flex items-center gap-1.5 font-mono text-[10.5px] text-success">
            <span className="inline-block w-1.5 h-1.5 rounded-full bg-success shadow-[0_0_8px_#16a34a]" />
            LIVE
          </div>
          <div className="font-mono text-[10px] text-muted uppercase tracking-[0.08em]">
            监控中
          </div>
        </div>
      </aside>

      {/* Mobile overlay */}
      {isMobile && mobileOpen && (
        <div
          onClick={() => setMobileOpen(false)}
          className="fixed inset-0 bg-black/60 z-[45]"
        />
      )}

      {/* Main */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Topbar */}
        <header className="flex items-center justify-between px-6 py-3 border-b border-border bg-bg/80 backdrop-blur-xl sticky top-0 z-30">
          <div className="flex items-center gap-3">
            {isMobile && (
              <button
                onClick={() => setMobileOpen(!mobileOpen)}
                className="bg-transparent border-none text-fg cursor-pointer p-1 rounded-sm"
              >
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                  <line x1="3" y1="6" x2="21" y2="6" />
                  <line x1="3" y1="12" x2="21" y2="12" />
                  <line x1="3" y1="18" x2="21" y2="18" />
                </svg>
              </button>
            )}
            <span className="font-body text-base text-fg tracking-[-0.01em]">
              {currentTitle}
            </span>
            <span className="font-mono text-[10px] text-success border border-success/30 rounded-pill px-2 py-0.5 bg-success/8">
              BETA
            </span>
          </div>
          <div className="flex items-center gap-3">
            <span className="font-mono text-[11px] text-muted">
              {time}
            </span>
          </div>
        </header>

        {/* Content */}
        <main className="flex-1 overflow-y-auto p-6 bg-bg">
          <Outlet />
        </main>
      </div>
    </div>
  )
}
