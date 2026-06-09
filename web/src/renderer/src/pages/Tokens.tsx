import { useState, useEffect, useCallback } from 'react'
import { tokenApi, type TokenInfo, type CreateTokenResponse } from '../api/tokens'
import { useToast } from '../components/Toast'
import Empty from '../components/Empty'

export default function Tokens() {
  const [tokens, setTokens] = useState<TokenInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [showModal, setShowModal] = useState(false)
  const [editingName, setEditingName] = useState('')
  const [editingExpiry, setEditingExpiry] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [revealToken, setRevealToken] = useState<CreateTokenResponse | null>(null)
  const toast = useToast()

  const load = useCallback(() => {
    tokenApi
      .list()
      .then(setTokens)
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    load()
  }, [load])

  function openModal() {
    setRevealToken(null)
    setEditingName('')
    setEditingExpiry('')
    setShowModal(true)
  }

  function closeModal() {
    setShowModal(false)
    setRevealToken(null)
    if (revealToken) load()
  }

  function handleOverlayClick(e: React.MouseEvent) {
    if (e.target === e.currentTarget) closeModal()
  }

  async function handleGenerate() {
    if (!editingName.trim()) {
      toast.error('请输入令牌名称')
      return
    }
    setSubmitting(true)
    try {
      const result = await tokenApi.create({
        name: editingName.trim(),
        expires_at: editingExpiry || null
      })
      setRevealToken(result)
      toast.success('令牌已生成')
    } catch {
      // error handled by interceptor
    } finally {
      setSubmitting(false)
    }
  }

  async function handleRevoke(token: TokenInfo) {
    if (!window.confirm(`确定要吊销令牌 "${token.name}" 吗？`)) return
    try {
      await tokenApi.revoke(token.id)
      toast.success('令牌已吊销')
      load()
    } catch {
      // error handled by interceptor
    }
  }

  async function handleCopy(text: string) {
    try {
      await window.electronAPI.clipboard.writeText(text)
      toast.info('令牌已复制')
    } catch {
      toast.error('复制失败')
    }
  }

  function formatDate(d: string | null): string {
    if (!d) return '永久'
    try {
      return new Date(d).toLocaleString('zh-CN', { hour12: false })
    } catch {
      return d
    }
  }

  function formatLastUsed(d: string | null): string {
    if (!d) return '—'
    try {
      return new Date(d).toLocaleString('zh-CN', { hour12: false })
    } catch {
      return d
    }
  }

  if (loading) {
    return <p style={{ color: 'var(--color-muted)', padding: 16 }}>加载中...</p>
  }

  return (
    <div>
      <div className="panel">
        <div className="panel-header">
          <span className="panel-title">API 令牌管理</span>
          <button className="btn btn-primary btn-sm" onClick={openModal}>
            + 生成令牌
          </button>
        </div>
        {tokens.length === 0 ? (
          <div style={{ padding: 24 }}>
            <Empty description="暂无 API 令牌" actionText="+ 生成令牌" onAction={openModal} />
          </div>
        ) : (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>名称</th>
                  <th>最后使用</th>
                  <th>过期时间</th>
                  <th>状态</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                {tokens.map((t) => (
                  <tr key={t.id}>
                    <td style={{ color: 'var(--color-fg)' }}>{t.name}</td>
                    <td className="mono">{formatLastUsed(t.last_used_at)}</td>
                    <td className="mono">{formatDate(t.expires_at)}</td>
                    <td>
                      <span className={t.revoked ? 'badge badge-danger' : 'badge badge-success'}>
                        {t.revoked ? '已吊销' : '有效'}
                      </span>
                    </td>
                    <td>
                      {t.revoked ? (
                        <span style={{ color: 'var(--color-meta)', fontSize: 13 }}>—</span>
                      ) : (
                        <div style={{ display: 'flex', gap: 8 }}>
                          <button
                            className="btn btn-ghost btn-sm"
                            onClick={() => handleCopy(t.token || '')}
                          >
                            复制
                          </button>
                          <button
                            className="btn btn-ghost btn-sm btn-danger"
                            onClick={() => handleRevoke(t)}
                          >
                            吊销
                          </button>
                        </div>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Generate Modal */}
      <div className={`modal-overlay${showModal ? ' open' : ''}`} onClick={handleOverlayClick}>
        <div className="modal">
          {revealToken ? (
            <>
              <h2>令牌已生成</h2>
              <div className="token-reveal">
                <code>{revealToken.token}</code>
              </div>
              <div className="field">
                <label>名称</label>
                <input value={revealToken.name} readOnly disabled />
              </div>
              <div className="field">
                <label>过期时间</label>
                <input value={formatDate(revealToken.expires_at)} readOnly disabled />
              </div>
              <div className="token-warning">
                <span>⚠</span>
                <span>请立即复制并安全保存此令牌，关闭后将无法再次查看</span>
              </div>
              <div className="modal-actions">
                <button
                  className="btn btn-primary btn-sm"
                  onClick={() => handleCopy(revealToken.token)}
                >
                  复制
                </button>
                <button className="btn btn-ghost btn-sm" onClick={closeModal}>
                  关闭
                </button>
              </div>
            </>
          ) : (
            <>
              <h2>生成 API 令牌</h2>
              <div className="field">
                <label>令牌名称 / 用途</label>
                <input
                  value={editingName}
                  onChange={(e) => setEditingName(e.target.value)}
                  placeholder="例如：生产环境监控"
                />
              </div>
              <div className="field">
                <label>过期时间（可选，留空为永久）</label>
                <input
                  type="date"
                  value={editingExpiry}
                  onChange={(e) => setEditingExpiry(e.target.value)}
                />
                <div className="field-help">不设置则令牌永久有效</div>
              </div>
              <div className="modal-actions">
                <button className="btn btn-ghost btn-sm" onClick={closeModal}>
                  取消
                </button>
                <button
                  className="btn btn-primary btn-sm"
                  disabled={submitting || !editingName.trim()}
                  onClick={handleGenerate}
                >
                  {submitting ? '生成中...' : '生成'}
                </button>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  )
}
