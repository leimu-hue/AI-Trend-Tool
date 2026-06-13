import { useState, useEffect, useCallback } from 'react'
import { queryApi, type Article } from '../api/queries'

const PER_PAGE = 20

export default function Articles() {
  const [articles, setArticles] = useState<Article[]>([])
  const [page, setPage] = useState(1)
  const [total, setTotal] = useState(0)
  const [loading, setLoading] = useState(true)

  const totalPages = Math.max(1, Math.ceil(total / PER_PAGE))

  const load = useCallback(async (p: number) => {
    setLoading(true)
    try {
      const result = await queryApi.getArticles({ page: p, per_page: PER_PAGE })
      setArticles(result.items)
      setTotal(result.total)
      setPage(result.page)
    } catch {
      // error handled by interceptor
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    load(1)
  }, [load])

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
          <span className="text-muted" style={{ fontSize: 13 }}>
            共 {total} 篇
          </span>
        </div>
        {loading ? (
          <p style={{ color: 'var(--color-muted)', padding: 16 }}>加载中...</p>
        ) : articles.length === 0 ? (
          <p style={{ color: 'var(--color-muted)', padding: 16 }}>暂无文章</p>
        ) : (
          <>
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>标题</th>
                    <th>来源</th>
                    <th>匹配关键词</th>
                    <th>发布时间</th>
                    <th>抓取时间</th>
                    <th>状态</th>
                  </tr>
                </thead>
                <tbody>
                  {articles.map((a) => (
                    <tr key={a.id}>
                      <td>
                        <a
                          href={a.link}
                          target="_blank"
                          rel="noopener noreferrer"
                          style={{ color: 'var(--color-accent)' }}
                        >
                          {a.title || a.link}
                        </a>
                      </td>
                      <td className="mono" style={{ fontSize: 12 }}>
                        {a.source_name}
                      </td>
                      <td>
                        {a.matched_keywords ? (
                          <span className="badge badge-info">{a.matched_keywords}</span>
                        ) : (
                          <span style={{ color: 'var(--color-meta)', fontSize: 13 }}>—</span>
                        )}
                      </td>
                      <td className="mono" style={{ fontSize: 12 }}>
                        {formatDate(a.published_at)}
                      </td>
                      <td className="mono" style={{ fontSize: 12 }}>
                        {formatDate(a.created_at)}
                      </td>
                      <td>
                        <span className={a.processed_at ? 'badge badge-success' : 'badge badge-warning'}>
                          {a.processed_at ? '已处理' : '未处理'}
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
                justifyContent: 'center',
                gap: 12,
                padding: '12px 0',
                borderTop: '1px solid var(--color-border)'
              }}
            >
              <button
                className="btn btn-ghost btn-sm"
                disabled={page <= 1}
                onClick={() => load(page - 1)}
              >
                上一页
              </button>
              <span style={{ color: 'var(--color-muted)', fontSize: 13 }}>
                第 {page} / {totalPages} 页
              </span>
              <button
                className="btn btn-ghost btn-sm"
                disabled={page >= totalPages}
                onClick={() => load(page + 1)}
              >
                下一页
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  )
}
