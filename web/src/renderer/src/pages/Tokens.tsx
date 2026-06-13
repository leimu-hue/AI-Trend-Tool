import { useState, useEffect, useCallback } from 'react'
import { DatePicker } from 'antd'
import dayjs, { type Dayjs } from 'dayjs'
import { tokenApi, type TokenInfo, type CreateTokenResponse } from '../api/tokens'
import { useToast } from '../components/Toast'
import Empty from '../components/Empty'
import Confirm from '../components/Confirm'

const EXPIRY_PRESETS = [
  { label: '7 天', days: 7 },
  { label: '30 天', days: 30 },
  { label: '90 天', days: 90 },
  { label: '1 年', days: 365 }
]

export default function Tokens() {
  const [tokens, setTokens] = useState<TokenInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [showModal, setShowModal] = useState(false)
  const [editingName, setEditingName] = useState('')
  const [editingExpiry, setEditingExpiry] = useState<Dayjs | null>(null)
  const [noExpiry, setNoExpiry] = useState(true)
  const [submitting, setSubmitting] = useState(false)
  const [revealToken, setRevealToken] = useState<CreateTokenResponse | null>(null)
  const [revokeTarget, setRevokeTarget] = useState<TokenInfo | null>(null)
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
    setEditingExpiry(null)
    setNoExpiry(true)
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
        expires_at: noExpiry || !editingExpiry ? null : editingExpiry.format('YYYY-MM-DDTHH:mm:ss')
      })
      setRevealToken(result)
      toast.success('令牌已生成')
    } catch {
      // error handled by interceptor
    } finally {
      setSubmitting(false)
    }
  }

  async function handleRevoke() {
    if (!revokeTarget) return
    try {
      await tokenApi.revoke(revokeTarget.id)
      toast.success('令牌已吊销')
      setRevokeTarget(null)
      load()
    } catch {
      // error handled by interceptor
    }
  }

  async function handleCopy(text: string) {
    if (!text) {
      toast.error('没有可复制的内容')
      return
    }
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
                        <button
                          className="btn btn-ghost btn-sm btn-danger"
                          onClick={() => setRevokeTarget(t)}
                        >
                          吊销
                        </button>
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
                <label>过期时间</label>
                <label className="expiry-checkbox">
                  <input
                    type="checkbox"
                    checked={noExpiry}
                    onChange={(e) => {
                      setNoExpiry(e.target.checked)
                      if (e.target.checked) setEditingExpiry(null)
                    }}
                  />
                  <span>永久有效</span>
                </label>
                {!noExpiry && (
                  <>
                    <div className="expiry-presets">
                      {EXPIRY_PRESETS.map((p) => (
                        <button
                          key={p.days}
                          type="button"
                          className="btn btn-ghost btn-sm expiry-preset-btn"
                          onClick={() => setEditingExpiry(dayjs().add(p.days, 'day'))}
                        >
                          {p.label}
                        </button>
                      ))}
                    </div>
                    <DatePicker
                      showTime={{ format: 'HH:mm' }}
                      format="YYYY-MM-DD HH:mm"
                      placeholder="选择过期时间"
                      value={editingExpiry}
                      onChange={(val) => setEditingExpiry(val)}
                      disabledDate={(current) => current && current < dayjs().startOf('day')}
                      style={{ width: '100%' }}
                      popupClassName="token-expiry-picker"
                    />
                  </>
                )}
                <div className="field-help">
                  {noExpiry ? '令牌将永久有效，直到手动吊销' : '也可点击上方快捷按钮快速设置'}
                </div>
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

      <Confirm
        open={revokeTarget !== null}
        title="吊销令牌"
        message={`确定要吊销令牌「${revokeTarget?.name}」吗？此操作不可撤销。`}
        confirmText="吊销"
        danger
        onConfirm={handleRevoke}
        onCancel={() => setRevokeTarget(null)}
      />
    </div>
  )
}
