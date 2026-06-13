import { useState, useEffect, useCallback } from 'react'
import { keywordApi, type Keyword } from '../api/keywords'
import { useToast } from '../components/Toast'
import Empty from '../components/Empty'
import Confirm from '../components/Confirm'

interface FormState {
  word: string
  case_sensitive: boolean
  std_multiplier: string
  min_hot_count: string
}

const EMPTY_FORM: FormState = {
  word: '',
  case_sensitive: false,
  std_multiplier: '2.0',
  min_hot_count: '3'
}

export default function Keywords() {
  const [keywords, setKeywords] = useState<Keyword[]>([])
  const [loading, setLoading] = useState(true)
  const [showModal, setShowModal] = useState(false)
  const [editingId, setEditingId] = useState<number | null>(null)
  const [form, setForm] = useState<FormState>(EMPTY_FORM)
  const [submitting, setSubmitting] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<Keyword | null>(null)
  const [deleting, setDeleting] = useState(false)
  const toast = useToast()

  const load = useCallback(() => {
    keywordApi
      .list()
      .then(setKeywords)
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    load()
  }, [load])

  function openAdd() {
    setEditingId(null)
    setForm(EMPTY_FORM)
    setShowModal(true)
  }

  function openEdit(item: Keyword) {
    setEditingId(item.id)
    setForm({
      word: item.word,
      case_sensitive: item.case_sensitive,
      std_multiplier: String(item.std_multiplier),
      min_hot_count: String(item.min_hot_count)
    })
    setShowModal(true)
  }

  function closeModal() {
    setShowModal(false)
    setEditingId(null)
  }

  function handleOverlayClick(e: React.MouseEvent) {
    if (e.target === e.currentTarget) closeModal()
  }

  function setField<K extends keyof FormState>(key: K, value: FormState[K]) {
    setForm((prev) => ({ ...prev, [key]: value }))
  }

  async function handleSubmit() {
    if (!form.word.trim()) {
      toast.error('请输入关键词')
      return
    }
    const stdMultiplier = form.std_multiplier === '' ? 2.0 : parseFloat(form.std_multiplier)
    const minHotCount = form.min_hot_count === '' ? 3 : parseInt(form.min_hot_count, 10)
    if (isNaN(stdMultiplier) || stdMultiplier <= 0) {
      toast.error('标准差倍数必须大于 0')
      return
    }
    if (isNaN(minHotCount) || minHotCount <= 0) {
      toast.error('最小触发计数必须 ≥ 1')
      return
    }
    setSubmitting(true)
    try {
      if (editingId !== null) {
        await keywordApi.update(editingId, {
          word: form.word.trim(),
          case_sensitive: form.case_sensitive,
          std_multiplier: stdMultiplier,
          min_hot_count: minHotCount
        })
        toast.success('关键词已更新')
      } else {
        await keywordApi.create({
          word: form.word.trim(),
          case_sensitive: form.case_sensitive,
          std_multiplier: stdMultiplier,
          min_hot_count: minHotCount
        })
        toast.success('关键词已添加')
      }
      closeModal()
      load()
    } catch {
      // error handled by interceptor
    } finally {
      setSubmitting(false)
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return
    setDeleting(true)
    try {
      await keywordApi.delete(deleteTarget.id)
      toast.success('关键词已删除')
      setDeleteTarget(null)
      load()
    } catch {
      // error handled by interceptor
    } finally {
      setDeleting(false)
    }
  }

  async function handleToggle(item: Keyword) {
    const action = item.enabled ? '暂停' : '启用'
    try {
      await keywordApi.update(item.id, { enabled: !item.enabled })
      toast.success(`关键词已${action}`)
      load()
    } catch {
      // error handled by interceptor
    }
  }

  if (loading) {
    return <p style={{ color: 'var(--color-muted)', padding: 16 }}>加载中...</p>
  }

  return (
    <div>
      <div className="panel">
        <div className="panel-header">
          <span className="panel-title">关键词管理</span>
          <button className="btn btn-primary btn-sm" onClick={openAdd}>
            + 添加关键词
          </button>
        </div>
        {keywords.length === 0 ? (
          <div style={{ padding: 24 }}>
            <Empty description="暂无关键词" actionText="+ 添加关键词" onAction={openAdd} />
          </div>
        ) : (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>关键词</th>
                  <th>大小写</th>
                  <th>标准差倍数</th>
                  <th>最小计数</th>
                  <th>24h 命中</th>
                  <th>状态</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                {keywords.map((k) => (
                  <tr key={k.id}>
                    <td className="mono" style={{ color: 'var(--color-fg)' }}>
                      {k.word}
                    </td>
                    <td>{k.case_sensitive ? '是' : '否'}</td>
                    <td>{k.std_multiplier}</td>
                    <td>{k.min_hot_count}</td>
                    <td className="mono" style={{ color: 'var(--color-success)' }}>
                      {k.hit_count_24h ?? '—'}
                    </td>
                    <td>
                      <span className={k.enabled ? 'badge badge-success' : 'badge badge-neutral'}>
                        {k.enabled ? '启用' : '暂停'}
                      </span>
                    </td>
                    <td>
                      <div style={{ display: 'flex', gap: 8 }}>
                        <button
                          className="btn btn-ghost btn-sm"
                          onClick={() => handleToggle(k)}
                        >
                          {k.enabled ? '暂停' : '启用'}
                        </button>
                        <button
                          className="btn btn-ghost btn-sm"
                          onClick={() => openEdit(k)}
                        >
                          编辑
                        </button>
                        <button
                          className="btn btn-ghost btn-sm btn-danger"
                          onClick={() => setDeleteTarget(k)}
                        >
                          删除
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Add/Edit Modal */}
      <div className={`modal-overlay${showModal ? ' open' : ''}`} onClick={handleOverlayClick}>
        <div className="modal">
          <h2>{editingId !== null ? '编辑关键词' : '添加关键词'}</h2>
          <div className="field">
            <label>关键词</label>
            <input
              value={form.word}
              onChange={(e) => setField('word', e.target.value)}
              placeholder="例如：AI 热点"
            />
          </div>
          <div className="field">
            <label>大小写敏感</label>
            <select
              value={form.case_sensitive ? '1' : '0'}
              onChange={(e) => setField('case_sensitive', e.target.value === '1')}
            >
              <option value="0">否</option>
              <option value="1">是</option>
            </select>
          </div>
          <div className="field">
            <label>标准差倍数</label>
            <input
              type="number"
              step="0.1"
              min="0"
              value={form.std_multiplier}
              onChange={(e) => setField('std_multiplier', e.target.value)}
            />
            <div className="field-help">当前计数超过均值+N倍标准差时触发热点</div>
          </div>
          <div className="field">
            <label>最小触发计数</label>
            <input
              type="number"
              step="1"
              min="1"
              value={form.min_hot_count}
              onChange={(e) => setField('min_hot_count', e.target.value)}
            />
            <div className="field-help">计数低于此值不计为热点</div>
          </div>
          <div className="modal-actions">
            <button className="btn btn-ghost btn-sm" onClick={closeModal}>
              取消
            </button>
            <button
              className="btn btn-primary btn-sm"
              disabled={submitting || !form.word.trim()}
              onClick={handleSubmit}
            >
              {submitting ? '提交中...' : editingId !== null ? '确认修改' : '确认添加'}
            </button>
          </div>
        </div>
      </div>

      <Confirm
        open={deleteTarget !== null}
        title="删除关键词"
        message={`确定要删除关键词「${deleteTarget?.word}」吗？此操作不可撤销。`}
        confirmText="删除"
        danger
        onConfirm={handleDelete}
        onCancel={() => setDeleteTarget(null)}
      />
    </div>
  )
}
