import { useState, useEffect, useCallback } from 'react'
import { Select } from 'antd'
import { queryApi, type Article, type Source } from '../api/queries'
import Empty from '../components/Empty'
import { useToast } from '../components/Toast'

const PER_PAGE = 20

export default function Articles() {
  const [articles, setArticles] = useState<Article[]>([])
  const [page, setPage] = useState(1)
  const [total, setTotal] = useState(0)
  const [loading, setLoading] = useState(true)

  // Filters
  const [sources, setSources] = useState<Source[]>([])
  const [sourceFilter, setSourceFilter] = useState<number | undefined>(undefined)
  const [processedFilter, setProcessedFilter] = useState<boolean | undefined>(undefined)

  const toast = useToast()

  const totalPages = Math.max(1, Math.ceil(total / PER_PAGE))

  // Build source name lookup
  const sourceNameMap = new Map(sources.map((s) => [s.id, s.name]))

  const load = useCallback(
    async (p: number, srcFilter: number | undefined, procFilter: boolean | undefined) => {
      setLoading(true)
      try {
        const result = await queryApi.getArticles({
          page: p,
          per_page: PER_PAGE,
          source_id: srcFilter,
          processed: procFilter,
        })
        setArticles(result.items)
        setTotal(result.total)
        setPage(result.page)
      } catch {
        // error handled by interceptor
      } finally {
        setLoading(false)
      }
    },
    []
  )

  // Load sources on mount
  useEffect(() => {
    queryApi.getSources().then(setSources).catch(() => setSources([]))
  }, [])

  // Load articles when filters change
  useEffect(() => {
    load(1, sourceFilter, processedFilter)
  }, [load, sourceFilter, processedFilter])

  // Trigger filter
  const handleRunFilter = useCallback(async () => {
    try {
      await queryApi.triggerFilter()
      toast.success('过滤器已触发，正在处理...')
      // Refresh list after trigger
      load(page, sourceFilter, processedFilter)
    } catch {
      // error handled by interceptor
    }
  }, [toast, load, page, sourceFilter, processedFilter])

  function formatDate(d: string | null): string {
    if (!d) return '—'
    try {
      return new Date(d).toLocaleString('zh-CN', { hour12: false })
    } catch {
      return d
    }
  }

  return (
    <div>
      <div className="panel">
        <div className="panel-header">
          <span className="panel-title">文章日志</span>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            {/* Source filter */}
            <Select
              className="filter-select"
              popupClassName="filter-select-dropdown"
              size="small"
              value={sourceFilter ?? ''}
              onChange={(val) => {
                setSourceFilter(val === '' ? undefined : Number(val))
              }}
              options={[
                { value: '', label: '全部数据源' },
                ...sources.map((s) => ({ value: s.id, label: s.name })),
              ]}
            />

            {/* Processed filter */}
            <Select
              className="filter-select"
              popupClassName="filter-select-dropdown"
              size="small"
              value={processedFilter === undefined ? '' : String(processedFilter)}
              onChange={(val) => {
                setProcessedFilter(val === '' ? undefined : val === 'true')
              }}
              options={[
                { value: '', label: '全部状态' },
                { value: 'false', label: '待处理' },
                { value: 'true', label: '已处理' },
              ]}
            />

            <button className="btn btn-ghost btn-sm" onClick={handleRunFilter}>
              运行过滤器
            </button>
          </div>
        </div>

        {loading ? (
          <p style={{ color: 'var(--color-muted)', padding: 16, textAlign: 'center' }}>加载中...</p>
        ) : articles.length === 0 ? (
          <Empty description="暂无文章" />
        ) : (
          <>
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>#</th>
                    <th>来源</th>
                    <th>标题</th>
                    <th>匹配关键词</th>
                    <th>发布时间</th>
                    <th>处理状态</th>
                  </tr>
                </thead>
                <tbody>
                  {articles.map((a) => (
                    <tr key={a.id}>
                      <td>
                        <span
                          className="mono"
                          style={{ fontSize: 11, color: 'var(--meta)' }}
                        >
                          {a.id}
                        </span>
                      </td>
                      <td>
                        <span className="mono" style={{ fontSize: 11 }}>
                          {sourceNameMap.get(a.source_id) || `#${a.source_id}`}
                        </span>
                      </td>
                      <td>
                        <a
                          href={a.link}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="truncate"
                          style={{
                            maxWidth: 340,
                            color: 'var(--color-accent)',
                          }}
                        >
                          {a.title || a.link || '(无标题)'}
                        </a>
                      </td>
                      <td>
                        <span
                          className="mono"
                          style={{ fontSize: 11, color: 'var(--color-success)' }}
                        >
                          —
                        </span>
                      </td>
                      <td>
                        <span className="mono" style={{ fontSize: 11 }}>
                          {formatDate(a.published_at)}
                        </span>
                      </td>
                      <td>
                        <span
                          className={`badge ${a.processed_at ? 'badge-success' : 'badge-warn'}`}
                        >
                          <span
                            className={`badge-dot dot-show ${a.processed_at ? 'success-dot' : 'warn-dot'}`}
                          />
                          {a.processed_at ? '已处理' : '待处理'}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {/* Pagination */}
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '12px 20px',
                borderTop: '1px solid var(--color-border)',
              }}
            >
              <span
                className="mono"
                style={{ fontSize: 11, color: 'var(--color-muted)' }}
              >
                共 {total} 条 · 第 {page}/{totalPages} 页
              </span>
              <div style={{ display: 'flex', gap: 8 }}>
                <button
                  className="btn btn-ghost btn-sm"
                  disabled={page <= 1}
                  onClick={() => load(page - 1, sourceFilter, processedFilter)}
                >
                  上一页
                </button>
                <button
                  className="btn btn-ghost btn-sm"
                  disabled={page >= totalPages}
                  onClick={() => load(page + 1, sourceFilter, processedFilter)}
                >
                  下一页
                </button>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  )
}
