import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { Card, Input, Button, Alert, Typography } from 'antd'
import { tokenApi } from '../api/tokens'

const { Title, Text } = Typography

export default function AuthPage() {
  const [token, setToken] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const navigate = useNavigate()

  useEffect(() => {
    const existing = localStorage.getItem('api_token')
    if (existing) {
      navigate('/dashboard')
    }
  }, [navigate])

  async function handleSubmit() {
    if (!token.trim()) return
    setLoading(true)
    setError('')
    localStorage.setItem('api_token', token.trim())
    try {
      await tokenApi.list()
      navigate('/dashboard')
    } catch {
      localStorage.removeItem('api_token')
      setError('Token 无效或已过期')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-bg">
      <Card
        className="w-[420px] max-w-[90vw] rounded-lg border border-border"
        styles={{ body: { padding: 32 } }}
      >
        <div className="text-center mb-6">
          <div className="w-10 h-10 rounded-[7px] bg-accent inline-flex items-center justify-center font-mono text-base text-fg mb-3">
            ◈
          </div>
          <Title level={3} className="!text-fg !m-0 !font-normal">
            AI 热点监控系统
          </Title>
          <Text className="!text-muted !text-xs !font-mono !uppercase !tracking-[0.08em]">
            请输入 API Token 以继续
          </Text>
        </div>

        {error && (
          <Alert
            type="error"
            message={error}
            className="mb-4"
            closable
            onClose={() => setError('')}
          />
        )}

        <Input.Password
          value={token}
          onChange={(e) => setToken(e.target.value)}
          onPressEnter={handleSubmit}
          placeholder="粘贴你的 API Token..."
          size="large"
          className="mb-4"
        />

        <Button
          type="primary"
          block
          size="large"
          loading={loading}
          onClick={handleSubmit}
        >
          {loading ? '验证中...' : '验证并进入'}
        </Button>

        <Text className="block mt-3 !text-meta text-[11px] text-center">
          初始 Token 在后端启动日志中查看
        </Text>
      </Card>
    </div>
  )
}
