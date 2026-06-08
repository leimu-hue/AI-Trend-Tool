import { Card } from 'antd'

export default function Articles() {
  return (
    <div>
      <Card
        title="文章日志"
        className="bg-surface rounded-md border border-border"
        classNames={{ header: 'border-border' }}
      >
        <p className="text-muted">文章日志内容即将上线</p>
      </Card>
    </div>
  )
}
