import type { Component } from 'vue'
import { defineAsyncComponent } from 'vue'

export interface ToolDef {
  id: string
  title: string
  group: string
  icon: string
  keywords: string[]
  status: 'ready' | 'planned'
  component: Component
}

/**
 * 工具注册表：新增工具 = 加一行 + 新建组件目录。
 * planned 状态的工具统一渲染占位页。
 */
export const TOOLS: ToolDef[] = [
  {
    id: 'json',
    title: 'JSON格式化',
    group: 'JSON',
    icon: 'braces',
    keywords: ['json', 'format', 'validate', 'minify', 'beautify', '格式化', '校验', '压缩'],
    status: 'ready',
    component: defineAsyncComponent(() => import('./json/JsonTool.vue')),
  },
  {
    id: 'json2md',
    title: 'JSON转Markdown',
    group: 'JSON',
    icon: 'file',
    keywords: ['json', 'markdown', 'md', 'table', 'doc', '转换', '表格', '文档'],
    status: 'ready',
    component: defineAsyncComponent(() => import('./json2md/Json2MdTool.vue')),
  },
  {
    id: 'yaml',
    title: 'JSON转YAML',
    group: 'JSON',
    icon: 'swap',
    keywords: ['json', 'yaml', 'yml', 'convert', '转换', '配置'],
    status: 'ready',
    component: defineAsyncComponent(() => import('./yaml/JsonYamlTool.vue')),
  },
  {
    id: 'timestamp',
    title: '时间戳',
    group: 'Time',
    icon: 'clock',
    keywords: ['timestamp', 'time', 'date', 'unix', 'epoch', '时间戳', '时间', '日期'],
    status: 'ready',
    component: defineAsyncComponent(() => import('./timestamp/TimestampTool.vue')),
  },
  {
    id: 'md5',
    title: 'MD5加密',
    group: 'Hash',
    icon: 'hash',
    keywords: ['md5', 'hash', 'digest', 'checksum', '加密', '哈希', '摘要'],
    status: 'ready',
    component: defineAsyncComponent(() => import('./hash/Md5Tool.vue')),
  },
  {
    id: 'base64',
    title: 'Base64',
    group: 'Codec',
    icon: 'code',
    keywords: ['base64', 'encode', 'decode', '编码', '解码', 'base'],
    status: 'ready',
    component: defineAsyncComponent(() => import('./base64/Base64Tool.vue')),
  },
  {
    id: 'unicode',
    title: 'Unicode',
    group: 'Codec',
    icon: 'globe',
    keywords: ['unicode', 'escape', '转义', '编码', '解码', 'u4f60'],
    status: 'ready',
    component: defineAsyncComponent(() => import('./unicode/UnicodeTool.vue')),
  },
  {
    id: 'password',
    title: '密码生成器',
    group: 'Generator',
    icon: 'key',
    keywords: ['password', 'generate', 'random', '密码', '生成', '随机'],
    status: 'ready',
    component: defineAsyncComponent(() => import('./password/PasswordTool.vue')),
  },
  {
    id: 'uuid',
    title: 'UUID生成器',
    group: 'Generator',
    icon: 'barcode',
    keywords: ['uuid', 'guid', 'generate', 'random', '唯一', '标识', '生成'],
    status: 'ready',
    component: defineAsyncComponent(() => import('./uuid/UuidTool.vue')),
  },
  {
    id: 'ip',
    title: '我的IP',
    group: 'Network',
    icon: 'globe',
    keywords: ['ip', 'ipv4', 'ipv6', 'address', 'location', 'geo', '我的ip', '公网ip', '归属地'],
    status: 'ready',
    component: defineAsyncComponent(() => import('./ip/IpTool.vue')),
  },
  {
    id: 'jwt',
    title: 'JWT解析',
    group: 'Codec',
    icon: 'shield',
    keywords: ['jwt', 'token', 'decode', 'json web token', '解析', '令牌', '解码'],
    status: 'ready',
    component: defineAsyncComponent(() => import('./jwt/JwtTool.vue')),
  },
]
