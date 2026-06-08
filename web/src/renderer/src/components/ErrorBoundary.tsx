import { Component, type ReactNode } from 'react'
import { Result, Button } from 'antd'

interface Props {
  children: ReactNode
}

interface State {
  hasError: boolean
  error: Error | null
}

export default class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = { hasError: false, error: null }
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: { componentStack: string }) {
    console.error('ErrorBoundary caught an error:', error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex items-center justify-center min-h-screen">
          <Result
            status="error"
            title="页面出错了"
            subTitle={this.state.error?.message || '未知错误'}
            extra={
              <Button type="primary" onClick={() => window.location.reload()}>
                刷新页面
              </Button>
            }
          />
        </div>
      )
    }
    return this.props.children
  }
}
