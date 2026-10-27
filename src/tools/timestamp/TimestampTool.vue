<script setup lang="ts">
import { computed, onActivated, onDeactivated, ref, watch } from 'vue'
import Icon from '../../components/Icon.vue'
import { clipboardWriteText } from '../../api'

const pad = (n: number) => String(n).padStart(2, '0')

/** 本地时区格式化：YYYY-MM-DD HH:mm:ss */
function formatDateTime(ms: number): string {
  const d = new Date(ms)
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

/** 10 位及以下按秒、11 位及以上按毫秒解析 */
function parseTimestamp(s: string): number | null {
  const t = s.trim()
  if (!/^-?\d+$/.test(t)) return null
  const n = Number(t)
  if (!Number.isFinite(n)) return null
  const ms = Math.abs(n) < 1e11 ? n * 1000 : n
  return Number.isNaN(new Date(ms).getTime()) ? null : ms
}

const DT_RE = /^(\d{4})-(\d{1,2})-(\d{1,2})(?:[ T](\d{1,2}):(\d{1,2})(?::(\d{1,2}))?)?$/

/** 解析日期时间字符串（本地时区），非法返回 null */
function parseDateTime(s: string): number | null {
  const m = DT_RE.exec(s.trim())
  if (!m) return null
  const y = Number(m[1])
  const mo = Number(m[2])
  const d = Number(m[3])
  const h = Number(m[4] ?? 0)
  const mi = Number(m[5] ?? 0)
  const se = Number(m[6] ?? 0)
  const dt = new Date(y, mo - 1, d, h, mi, se)
  // 校验溢出（如 2 月 30 日、25 点）
  if (dt.getFullYear() !== y || dt.getMonth() !== mo - 1 || dt.getDate() !== d) return null
  if (dt.getHours() !== h || dt.getMinutes() !== mi || dt.getSeconds() !== se) return null
  return dt.getTime()
}

// ---------- 状态 ----------

const nowMs = ref(Date.now())
const nowSeconds = computed(() => Math.floor(nowMs.value / 1000))
const tsInput = ref(String(Math.floor(Date.now() / 1000)))
const dtInput = ref(formatDateTime(Date.now()))
const tsCopied = ref(false)
const dtCopied = ref(false)
/** 交换两个区块的显示顺序 */
const swapped = ref(false)

const tsMs = computed(() => parseTimestamp(tsInput.value))
const dtMs = computed(() => parseDateTime(dtInput.value))

const tsInvalid = computed(() => tsInput.value.trim() !== '' && tsMs.value == null)
const dtInvalid = computed(() => dtInput.value.trim() !== '' && dtMs.value == null)

const isSeconds = computed(() => {
  const t = tsInput.value.trim()
  return /^-?\d+$/.test(t) && Math.abs(Number(t)) < 1e11
})

/** 时间戳结果：按输入单位回显（秒/毫秒） */
const tsDisplay = computed(() => {
  const ms = tsMs.value
  if (ms == null) return ''
  return isSeconds.value ? String(Math.trunc(ms / 1000)) : String(ms)
})

const dtDisplay = computed(() => {
  const ms = dtMs.value
  return ms == null ? '' : formatDateTime(ms)
})

// ---------- 双向联动 ----------

let syncing = false

watch(
  tsInput,
  (v) => {
    if (syncing) return
    const ms = parseTimestamp(v)
    if (ms == null) return
    syncing = true
    dtInput.value = formatDateTime(ms)
    syncing = false
  },
  { flush: 'sync' },
)

watch(
  dtInput,
  (v) => {
    if (syncing) return
    const ms = parseDateTime(v)
    if (ms == null) return
    // 保留用户当前使用的单位（秒输入仍输出秒）
    syncing = true
    tsInput.value = isSeconds.value ? String(Math.trunc(ms / 1000)) : String(ms)
    syncing = false
  },
  { flush: 'sync' },
)

// ---------- 操作 ----------

function useNowTimestamp() {
  tsInput.value = String(Math.floor(Date.now() / 1000))
}

function useNowDateTime() {
  dtInput.value = formatDateTime(Date.now())
}

type Preset = 'now' | 'today' | 'monday' | 'month' | 'year'

function applyPreset(kind: Preset) {
  const d = new Date()
  if (kind !== 'now') {
    d.setHours(0, 0, 0, 0)
    if (kind === 'monday') d.setDate(d.getDate() - ((d.getDay() + 6) % 7))
    else if (kind === 'month') d.setDate(1)
    else if (kind === 'year') d.setMonth(0, 1)
  }
  tsInput.value = String(Math.trunc(d.getTime() / 1000))
}

async function copyTs() {
  if (!tsDisplay.value) return
  await clipboardWriteText(tsDisplay.value)
  tsCopied.value = true
  setTimeout(() => (tsCopied.value = false), 1500)
}

async function copyDt() {
  if (!dtDisplay.value) return
  await clipboardWriteText(dtDisplay.value)
  dtCopied.value = true
  setTimeout(() => (dtCopied.value = false), 1500)
}

// ---------- 当前时间刷新 ----------

let timer: number | undefined

function startTimer() {
  nowMs.value = Date.now()
  window.clearInterval(timer)
  timer = window.setInterval(() => (nowMs.value = Date.now()), 1000)
}

function stopTimer() {
  window.clearInterval(timer)
  timer = undefined
}

// 工具组件被 KeepAlive 缓存，不会 unmount，只能在停用/激活时开关定时器
onActivated(startTimer)
onDeactivated(stopTimer)
</script>

<template>
  <div class="tool-body">
    <div class="ts-body">
      <!-- 时间戳 -->
      <section class="ts-block" :style="{ order: swapped ? 2 : 0 }">
        <div class="ts-head">
          <h2>时间戳</h2>
          <span class="ts-now">当前时间: {{ nowSeconds }}</span>
        </div>
        <div class="ts-field" :class="{ invalid: tsInvalid }">
          <input
            v-model="tsInput"
            class="ts-input"
            spellcheck="false"
            placeholder="1793000097 (seconds) / 17930000970 (milliseconds)"
          />
          <button class="ts-icon-btn" title="Use current time" @click="useNowTimestamp()">
            <Icon name="refresh" />
          </button>
        </div>
        <div class="ts-presets">
          <span class="ts-preset-label">常用时间戳:</span>
          <button class="ts-chip" @click="applyPreset('now')">当前时间</button>
          <button class="ts-chip" @click="applyPreset('today')">今日零点</button>
          <button class="ts-chip" @click="applyPreset('monday')">本周一</button>
          <button class="ts-chip" @click="applyPreset('month')">本月1</button>
          <button class="ts-chip" @click="applyPreset('year')">今年初</button>
        </div>
        <div class="ts-field readonly">
          <span class="ts-result" :class="{ empty: !tsDisplay }">{{ tsDisplay || '—' }}</span>
          <button
            class="ts-icon-btn"
            :title="tsCopied ? 'Copied!' : 'Copy timestamp'"
            :disabled="!tsDisplay"
            @click="copyTs()"
          >
            <Icon :name="tsCopied ? 'check' : 'copy'" />
          </button>
        </div>
      </section>

      <!-- 交换按钮（位于两块之间） -->
      <div class="ts-swap-row" style="order: 1">
        <button class="ts-swap" title="Swap sections" @click="swapped = !swapped">
          <Icon name="swap" />
        </button>
      </div>

      <!-- 日期时间 -->
      <section class="ts-block" :style="{ order: swapped ? 0 : 2 }">
        <div class="ts-head">
          <h2>日期时间</h2>
        </div>
        <div class="ts-field" :class="{ invalid: dtInvalid }">
          <input
            v-model="dtInput"
            class="ts-input"
            spellcheck="false"
            placeholder="2026-10-26 15:49:30"
          />
          <button class="ts-icon-btn" title="Use current time" @click="useNowDateTime()">
            <Icon name="refresh" />
          </button>
          <button
            class="ts-icon-btn"
            :title="dtCopied ? 'Copied!' : 'Copy date time'"
            :disabled="!dtInput.trim() || dtInvalid"
            @click="copyDt()"
          >
            <Icon :name="dtCopied ? 'check' : 'copy'" />
          </button>
        </div>
      </section>
    </div>
  </div>
</template>
