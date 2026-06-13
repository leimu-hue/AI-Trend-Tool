import { useEffect, useState, useMemo, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import ReactECharts from 'echarts-for-react'
import * as echarts from 'echarts'
import { queryApi, type HotEvent, type Article, type Source, type Keyword, type TrendPoint } from '../api/queries'
import Empty from '../components/Empty'
import { useToast } from '../components/Toast'

function formatDate(d: string | null): string {
  if (!d) return '—'
  try {
    return new Date(d).toLocaleString('zh-CN', { hour12: false })
  } catch {
    return d
  }
}

export default function Dashboard() {
  const [sources, setSources] = useState<Source[]>([])
  const [keywords, setKeywords] = useState<Keyword[]>([])
  const [hotspots, setHotspots] = useState<HotEvent[]>([])
  const [articles, setArticles] = useState<Article[]>([])
  const [pushStatusMap, setPushStatusMap] = useState<Record<number, string>>({})
  const [trendPoints, setTrendPoints] = useState<TrendPoint[]>([])
  const [activeKeyword, setActiveKeyword] = useState('')
  const [activeKeywordId, setActiveKeywordId] = useState<number | null>(null)
  const [loading, setLoading] = useState(true)

  const navigate = useNavigate()
  const toast = useToast()

  // Build keyword lookup map
  const keywordMap = useMemo(() => {
    const map: Record<number, Keyword> = {}
    keywords.forEach((kw) => { map[kw.id] = kw })
    return map
  }, [keywords])

  // Build source lookup map
  const sourceMap = useMemo(() => {
    const map: Record<number, string> = {}
    sources.forEach((s) => { map[s.id] = s.name })
    return map
  }, [sources])

  // Compute deviation (z-score)
  function calcDeviation(h: HotEvent): number {
    if (h.stddev_historical <= 0) return 0
    return (h.count - h.mean_historical) / h.stddev_historical
  }

  function getSeverityClass(deviation: number): string {
    if (deviation >= 3.0) return 'critical'
    if (deviation >= 2.0) return 'high'
    return 'medium'
  }

  // Fetch all data on mount
  useEffect(() => {
    let cancelled = false

    async function loadAll() {
      setLoading(true)
      try {
        const [hotspotsRes, articlesRes, sourcesRes, keywordsRes] = await Promise.allSettled([
          queryApi.getHotspots({ per_page: 50 }),
          queryApi.getArticles({ per_page: 5 }),
          queryApi.getSources(),
          queryApi.getKeywords(),
        ])

        if (cancelled) return

        if (hotspotsRes.status === 'fulfilled') {
          const hs = hotspotsRes.value
          setHotspots(hs.items)

          // Fetch push records for all hotspots in parallel
          if (hs.items.length > 0) {
            const pushResults = await Promise.allSettled(
              hs.items.map((h) => queryApi.getPushRecords(h.id))
            )
            const statusMap: Record<number, string> = {}
            pushResults.forEach((pr, idx) => {
              const heId = hs.items[idx].id
              if (pr.status === 'fulfilled') {
                const hasSent = pr.value.some((r) => r.status === 'sent')
                statusMap[heId] = hasSent ? 'sent' : 'pending'
              } else {
                statusMap[heId] = 'unknown'
              }
            })
            setPushStatusMap(statusMap)

            // Auto-load trend for first hotspot
            if (!cancelled && hs.items.length > 0) {
              const first = hs.items[0]
              const kw = keywordMap[first.keyword_id]
              if (kw) {
                try {
                  const trend = await queryApi.getTrend(kw.id, 24)
                  setTrendPoints(trend.points)
                  setActiveKeyword(kw.word)
                  setActiveKeywordId(kw.id)
                } catch { /* trend load failure is non-fatal */ }
              }
            }
          }
        } else {
          setHotspots([])
        }

        if (articlesRes.status === 'fulfilled') {
          setArticles(articlesRes.value.items)
        } else {
          setArticles([])
        }

        if (sourcesRes.status === 'fulfilled') {
          setSources(sourcesRes.value)
        } else {
          setSources([])
        }

        if (keywordsRes.status === 'fulfilled') {
          setKeywords(keywordsRes.value)
        } else {
          setKeywords([])
        }
      } finally {
        if (!cancelled) setLoading(false)
      }
    }

    loadAll()
    return () => { cancelled = true }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  // Click hotspot row → load trend
  const handleHotspotClick = useCallback(async (h: HotEvent) => {
    const kw = keywordMap[h.keyword_id]
    if (!kw) return
    try {
      const trend = await queryApi.getTrend(kw.id, 24)
      setTrendPoints(trend.points)
      setActiveKeyword(kw.word)
      setActiveKeywordId(kw.id)
    } catch {
      // error handled by interceptor
    }
  }, [keywordMap])

  // Manual filter trigger
  const handleTriggerFilter = useCallback(async () => {
    try {
      await queryApi.triggerFilter()
      toast.success('过滤器已触发，正在处理...')
    } catch {
      // error handled by interceptor
    }
  }, [toast])

  // Build ECharts option
  const chartOption = useMemo(() => {
    const labels = trendPoints.map((p) => p.hour_bucket)
    const data = trendPoints.map((p) => p.count)

    return {
      backgroundColor: 'transparent',
      grid: { left: 44, right: 20, top: 16, bottom: 28 },
      xAxis: {
        type: 'category' as const,
        data: labels,
        axisLabel: {
          color: '#868584',
          fontFamily: 'ui-monospace, SF Mono, Menlo, Monaco, Consolas, monospace',
          fontSize: 10,
        },
        axisLine: { lineStyle: { color: 'rgba(255,255,255,0.05)' } },
        axisTick: { show: false },
      },
      yAxis: {
        type: 'value' as const,
        minInterval: 1,
        axisLabel: {
          color: '#868584',
          fontFamily: 'ui-monospace, SF Mono, Menlo, Monaco, Consolas, monospace',
          fontSize: 10,
        },
        splitLine: { lineStyle: { color: 'rgba(255,255,255,0.03)' } },
      },
      series: [
        {
          type: 'line' as const,
          data,
          smooth: true,
          symbol: 'circle',
          symbolSize: 4,
          lineStyle: { color: '#16a34a', width: 2 },
          areaStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: 'rgba(22,163,74,0.25)' },
              { offset: 1, color: 'rgba(22,163,74,0)' },
            ]),
          },
          itemStyle: { color: '#16a34a' },
        },
      ],
      tooltip: {
        trigger: 'axis' as const,
        backgroundColor: 'rgba(31,29,27,0.95)',
        borderColor: 'rgba(226,226,226,0.2)',
        textStyle: { color: '#faf9f6', fontSize: 12 },
      },
    }
  }, [trendPoints])

  // Stat card data
  const sourcesEnabled = sources.filter((s) => s.enabled).length
  const keywordsEnabled = keywords.filter((k) => k.enabled).length
  const todayArticles = articles.length // approximation; backend lacks today-specific endpoint
  const activeHotspots = hotspots.length

  if (loading) {
    return (
      <div style={{ color: 'var(--color-muted)', padding: 48, textAlign: 'center', fontSize: 14 }}>
        加载中...
      </div>
    )
  }

  return (
    <div>
      {/* Stat cards */}
      <div className="stats-row">
        <div className="stat-card">
          <div className="stat-label">数据源</div>
          <div className="stat-value">{sources.length}</div>
          <div className="stat-sub">{sourcesEnabled} 个已启用</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">关键词</div>
          <div className="stat-value">{keywords.length}</div>
          <div className="stat-sub">{keywordsEnabled} 个已启用</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">今日文章</div>
          <div className="stat-value">{todayArticles}</div>
          <div className={`stat-sub${activeHotspots > 0 ? ' up' : ''}`}>
            最新 {todayArticles} 篇
          </div>
        </div>
        <div className="stat-card">
          <div className="stat-label">活跃热点</div>
          <div className="stat-value">{activeHotspots}</div>
          <div className={`stat-sub${activeHotspots > 0 ? ' warn' : ''}`}>
            {activeHotspots > 0 ? `发现 ${activeHotspots} 个热点` : '暂无热点'}
          </div>
        </div>
      </div>

      {/* Dual column: trend chart + hotspot table */}
      <div className="grid-2-wide">
        {/* Trend chart panel */}
        <div className="panel">
          <div className="panel-header">
            <span className="panel-title">关键词趋势（24h）</span>
            {activeKeyword && (
              <span className="badge badge-success">{activeKeyword}</span>
            )}
          </div>
          <div className="trend-chart">
            {trendPoints.length > 0 ? (
              <ReactECharts option={chartOption} style={{ height: 240 }} />
            ) : (
              <div style={{ padding: 48, textAlign: 'center', color: 'var(--color-muted)', fontSize: 13 }}>
                暂无趋势数据
              </div>
            )}
          </div>
        </div>

        {/* Hotspot table panel */}
        <div className="panel">
          <div className="panel-header">
            <span className="panel-title">活跃热点</span>
            <button className="btn btn-primary btn-sm" onClick={handleTriggerFilter}>
              手动扫描
            </button>
          </div>
          {hotspots.length === 0 ? (
            <Empty description="暂无热点事件" />
          ) : (
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>关键词</th>
                    <th>热度</th>
                    <th>偏离</th>
                    <th>状态</th>
                  </tr>
                </thead>
                <tbody>
                  {hotspots.map((h) => {
                    const kw = keywordMap[h.keyword_id]
                    const dev = calcDeviation(h)
                    const pushStatus = pushStatusMap[h.id] || 'pending'
                    const isSelected = activeKeywordId === h.keyword_id
                    return (
                      <tr
                        key={h.id}
                        onClick={() => handleHotspotClick(h)}
                        style={{
                          cursor: 'pointer',
                          background: isSelected ? 'rgba(22,163,74,0.06)' : undefined,
                        }}
                      >
                        <td style={{ color: 'var(--color-fg)', fontWeight: 400 }}>
                          {kw?.word || `#${h.keyword_id}`}
                        </td>
                        <td className="mono" style={{ fontSize: 12 }}>
                          {h.count} 次/小时
                        </td>
                        <td>
                          <span className={`severity ${getSeverityClass(dev)}`}>
                            {dev.toFixed(1)}σ
                          </span>
                        </td>
                        <td>
                          <span className={`badge ${pushStatus === 'sent' ? 'badge-success' : 'badge-warn'}`}>
                            {pushStatus === 'sent' ? '已推送' : '待推送'}
                          </span>
                        </td>
                      </tr>
                    )
                  })}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>

      {/* Recent articles panel */}
      <div className="panel">
        <div className="panel-header">
          <span className="panel-title">最新文章</span>
          <button className="btn btn-ghost btn-sm" onClick={() => navigate('/articles')}>
            查看全部 →
          </button>
        </div>
        {articles.length === 0 ? (
          <Empty description="暂无文章" />
        ) : (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>来源</th>
                  <th>标题</th>
                  <th>匹配关键词</th>
                  <th>发布时间</th>
                  <th>状态</th>
                </tr>
              </thead>
              <tbody>
                {articles.map((a) => (
                  <tr key={a.id}>
                    <td className="mono" style={{ fontSize: 11 }}>
                      {sourceMap[a.source_id] || `#${a.source_id}`}
                    </td>
                    <td>
                      <a
                        href={a.link}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="truncate"
                        style={{ maxWidth: 320, color: 'var(--fg)' }}
                      >
                        {a.title || a.link || '(无标题)'}
                      </a>
                    </td>
                    <td>
                      <span className="mono" style={{ fontSize: 11, color: 'var(--color-success)' }}>
                        —
                      </span>
                    </td>
                    <td className="mono" style={{ fontSize: 11 }}>
                      {formatDate(a.published_at)}
                    </td>
                    <td>
                      <span className={`badge ${a.processed_at ? 'badge-success' : 'badge-warn'}`}>
                        {a.processed_at ? '已处理' : '待处理'}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  )
}
