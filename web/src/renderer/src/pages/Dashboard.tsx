import { Card } from 'antd'

export default function Dashboard() {
  return (
    <div>
      <Card
        title="仪表盘"
        className="bg-surface rounded-md border border-border"
        classNames={{
          header: 'border-border',
          body: ''
        }}
      >
        <p className="text-muted">仪表盘内容即将上线</p>
      </Card>
    </div>
  )
}
