<script setup lang="ts">
import { computed, onActivated, onDeactivated, ref, watch } from 'vue'
import Icon from '../../components/Icon.vue'
import { clipboardWriteText, jwtDecode, type JwtDecoded } from '../../api'

/** 示例令牌的头部与载荷（签名是占位串，仅用于演示解码） */
const SAMPLE_HEADER = { alg: 'HS256', typ: 'JWT' }
const SAMPLE_PAYLOAD = { sub: '1234567890', name: '演示用户' }
/** 示例有效期：1 小时 50 分钟 */
const SAMPLE_TTL = 6600
const SAMPLE_SIGNATURE = 'SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c'

type TabKey = 'header' | 'payload' | 'signature'

const TABS: Array<{ key: TabKey; label: string }> = [
  { key: 'header', label: '头部 (Header)' },
  { key: 'payload', label: '载荷 (Payload)' },
  { key: 'signature', label: '签名 (Signature)' },
]

const token = ref('')
const decoded = ref<JwtDecoded | null>(null)
const tab = ref<TabKey>('header')
const showSegments = ref(false)
const copiedTab = ref(false)
const copiedToken = ref(false)
/** 每秒跳动的时间基准，用于实时刷新“剩余时间” */
const now = ref(Math.floor(Date.now() / 1000))
let timer: number | undefined

let seq = 0
let debounce: number | undefined

async function parse() {
  const mySeq = ++seq
  const text = token.value.trim()
  if (!text) {
    decoded.value = null
    return
  }
  const res = await jwtDecode(text)
  if (mySeq === seq) decoded.value = res
}

watch(token, () => {
  window.clearTimeout(debounce)
  debounce = window.setTimeout(parse, 120)
})

const tabContent = computed(() => {
  const d = decoded.value
  if (!d || !d.valid) return ''
  if (tab.value === 'header') return d.header ?? ''
  if (tab.value === 'payload') return d.payload ?? ''
  return d.signature ?? ''
})

const segments = computed(() => {
  const parts = token.value.trim().split('.')
  if (parts.length !== 3) return []
  return [
    { label: '头部', text: parts[0], cls: 'seg-header' },
    { label: '载荷', text: parts[1], cls: 'seg-payload' },
    { label: '签名', text: parts[2], cls: 'seg-signature' },
  ]
})

function pad(n: number): string {
  return String(n).padStart(2, '0')
}

/** Unix 秒 → 2026/10/26 17:17:07 */
function fmtTime(sec: number | null): string {
  if (sec == null) return '—'
  const d = new Date(sec * 1000)
  return `${d.getFullYear()}/${pad(d.getMonth() + 1)}/${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

/** 秒数 → 1小时 2分钟 / 3分钟 4秒 / 5秒 */
function fmtDuration(total: number): string {
  const s = Math.max(0, total)
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  const sec = s % 60
  if (h > 0) return m > 0 ? `${h}小时 ${m}分钟` : `${h}小时`
  if (m > 0) return `${m}分钟 ${sec}秒`
  return `${sec}秒`
}

const statusText = computed(() => {
  const d = decoded.value
  if (!d) return ''
  return d.valid ? '格式有效' : '格式无效'
})

const statusError = computed(() => {
  const d = decoded.value
  return d && !d.valid ? (d.error ?? '解析失败') : ''
})

const expiry = computed(() => {
  const d = decoded.value
  if (!d || !d.valid) return { text: '—', kind: 'muted' as const }
  if (d.expiresAt == null) return { text: '无 exp 声明', kind: 'muted' as const }
  const diff = d.expiresAt - now.value
  if (diff > 0) return { text: `有效，剩余 ${fmtDuration(diff)}`, kind: 'ok' as const }
  return { text: `已过期 ${fmtDuration(-diff)}`, kind: 'err' as const }
})

const issuedText = computed(() => fmtTime(decoded.value?.issuedAt ?? null))

async function copyTab() {
  if (!tabContent.value) return
  await clipboardWriteText(tabContent.value)
  copiedTab.value = true
  setTimeout(() => (copiedTab.value = false), 1500)
}

async function copyToken() {
  if (!token.value.trim()) return
  await clipboardWriteText(token.value.trim())
  copiedToken.value = true
  setTimeout(() => (copiedToken.value = false), 1500)
}

/** 示例令牌缓存：未过期时复用，过期后按当前时间重新签发 */
let sampleCache: { token: string; exp: number } | null = null

function b64url(text: string): string {
  const bytes = new TextEncoder().encode(text)
  let bin = ''
  bytes.forEach((b) => (bin += String.fromCharCode(b)))
  return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

function buildSampleToken(): string {
  const nowSec = Math.floor(Date.now() / 1000)
  if (sampleCache && sampleCache.exp > nowSec) return sampleCache.token

  const iat = nowSec
  const exp = iat + SAMPLE_TTL
  const payload = { ...SAMPLE_PAYLOAD, iat, exp }
  const sample = [
    b64url(JSON.stringify(SAMPLE_HEADER)),
    b64url(JSON.stringify(payload)),
    SAMPLE_SIGNATURE,
  ].join('.')
  sampleCache = { token: sample, exp }
  return sample
}

function loadSample() {
  token.value = buildSampleToken()
}

function clearAll() {
  token.value = ''
  decoded.value = null
}

function startTimer() {
  now.value = Math.floor(Date.now() / 1000)
  window.clearInterval(timer)
  timer = window.setInterval(() => (now.value = Math.floor(Date.now() / 1000)), 1000)
}

function stopTimer() {
  window.clearInterval(timer)
  timer = undefined
}

onActivated(startTimer)
onDeactivated(stopTimer)
</script>

<template>
  <div class="tool-body">
    <div class="jwt-body">
      <!-- 左：令牌输入与状态 -->
      <section class="jwt-panel">
        <h2 class="jwt-title">解码JWT令牌</h2>

        <div class="jwt-field-head">
          <span class="jwt-label">JWT令牌:</span>
          <button class="jwt-link" @click="showSegments = !showSegments">
            {{ showSegments ? '隐藏分段' : '显示分段' }}
          </button>
        </div>

        <textarea
          v-model="token"
          class="jwt-textarea"
          spellcheck="false"
          placeholder="粘贴 JWT 令牌，例如 eyJhbGciOi..."
        />

        <div v-if="showSegments && segments.length" class="jwt-segments">
          <div v-for="s in segments" :key="s.label" class="jwt-segment" :class="s.cls">
            <span class="jwt-segment-label">{{ s.label }}</span>
            <code>{{ s.text }}</code>
          </div>
        </div>

        <div class="jwt-btns">
          <button class="hash-compute" @click="loadSample()">
            <Icon name="flask" />
            <span>加载示例</span>
          </button>
          <button class="pwd-btn" @click="clearAll()">
            <Icon name="trash" />
            <span>清空</span>
          </button>
          <button class="pwd-btn" :disabled="!token.trim()" @click="copyToken()">
            <Icon :name="copiedToken ? 'check' : 'copy'" />
            <span>复制令牌</span>
          </button>
        </div>

        <dl class="jwt-meta">
          <div class="jwt-meta-row">
            <dt>JWT状态:</dt>
            <dd :class="decoded?.valid ? 'ok' : decoded ? 'err' : 'muted'">
              {{ statusText || '—' }}
            </dd>
          </div>
          <div v-if="statusError" class="jwt-meta-row">
            <dt></dt>
            <dd class="err jwt-meta-detail">{{ statusError }}</dd>
          </div>
          <div class="jwt-meta-row">
            <dt>过期状态:</dt>
            <dd :class="expiry.kind">{{ expiry.text }}</dd>
          </div>
          <div class="jwt-meta-row">
            <dt>签发时间:</dt>
            <dd class="muted">{{ issuedText }}</dd>
          </div>
        </dl>
      </section>

      <!-- 右：解析结果 -->
      <section class="jwt-panel">
        <div class="jwt-tabs" role="tablist">
          <button
            v-for="t in TABS"
            :key="t.key"
            class="jwt-tab"
            :class="{ active: tab === t.key }"
            role="tab"
            :aria-selected="tab === t.key"
            @click="tab = t.key"
          >
            {{ t.label }}
          </button>
          <span class="spacer" />
          <button
            class="ts-icon-btn"
            :title="copiedTab ? 'Copied!' : 'Copy'"
            :disabled="!tabContent"
            @click="copyTab()"
          >
            <Icon :name="copiedTab ? 'check' : 'copy'" />
          </button>
        </div>

        <pre v-if="tabContent" class="jwt-output">{{ tabContent }}</pre>
        <div v-else class="jwt-output-placeholder">
          {{ decoded && !decoded.valid ? '令牌解析失败' : '粘贴 JWT 令牌后自动解析' }}
        </div>
      </section>
    </div>
  </div>
</template>
