import { Card } from 'antd'

export default function Settings() {
  return (
    <div className="grid grid-cols-2 gap-4">
      <Card
        title="API 令牌管理"
        className="bg-surface rounded-md border border-border"
        classNames={{ header: 'border-border' }}
      >
        <p className="text-muted">设置页面内容即将上线</p>
      </Card>
      <Card
        title="系统配置"
        className="bg-surface rounded-md border border-border"
        classNames={{ header: 'border-border' }}
      >
        <p className="text-muted">设置页面内容即将上线</p>
      </Card>
    </div>
  )
}
