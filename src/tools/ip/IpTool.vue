<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import Icon from '../../components/Icon.vue'
import { clipboardWriteText, ipLookup, type IpInfo, type IpReport } from '../../api'

type Family = 'ipv4' | 'ipv6'

const report = ref<IpReport | null>(null)
const active = ref<Family>('ipv4')
const busy = ref(false)
const errorMsg = ref('')
const copied = ref(false)

const info = computed<IpInfo | null>(() => report.value?.[active.value] ?? null)
const other = computed<IpInfo | null>(() =>
  active.value === 'ipv4' ? (report.value?.ipv6 ?? null) : (report.value?.ipv4 ?? null),
)

/** 当前网络没有对应的地址族，或该地址族查询失败 */
const missingHint = computed(() => {
  if (!report.value || report.value[active.value]) return ''
  const label = active.value === 'ipv4' ? 'IPv4' : 'IPv6'
  return `当前网络未获取到 ${label} 地址`
})

async function refresh() {
  if (busy.value) return
  busy.value = true
  errorMsg.value = ''
  try {
    const data = await ipLookup()
    report.value = data
    // 默认展示 IPv4，没有则展示 IPv6
    if (!data.ipv4) active.value = 'ipv6'
    if (!data.ipv6 && data.ipv4) active.value = 'ipv4'
  } catch (e) {
    report.value = null
    errorMsg.value = String(e)
  } finally {
    busy.value = false
  }
}

function switchTo(f: Family) {
  if (!report.value?.[f]) return
  active.value = f
  copied.value = false
}

async function copyIp() {
  if (!info.value?.ip) return
  await clipboardWriteText(info.value.ip)
  copied.value = true
  setTimeout(() => (copied.value = false), 1500)
}

function dash(v: string | number | null | undefined): string {
  return v == null || v === '' ? '—' : String(v)
}

/** 省市区同名时只保留一份，例如「中国 · 北京」 */
const locationText = computed(() => {
  const i = info.value
  if (!i) return ''
  const parts = [i.country, i.region, i.city].filter((p): p is string => !!p)
  const unique = parts.filter((p, idx) => parts.indexOf(p) === idx)
  return unique.join(' · ')
})

const countryText = computed(() => {
  const i = info.value
  if (!i) return '—'
  if (i.country && i.countryCode && i.country !== i.countryCode) {
    return `${i.country} (${i.countryCode})`
  }
  return dash(i.country ?? i.countryCode)
})

const coordText = computed(() => {
  const i = info.value
  if (!i || i.latitude == null || i.longitude == null) return '—'
  return `${i.latitude.toFixed(4)}, ${i.longitude.toFixed(4)}`
})

const timezoneText = computed(() => {
  const i = info.value
  if (!i) return '—'
  if (i.timezone && i.utcOffset) return `${i.timezone} (UTC${i.utcOffset})`
  return dash(i.timezone ?? i.utcOffset)
})

const geoRows = computed(() => {
  const i = info.value
  if (!i) return []
  return [
    { label: '国家 / 地区', value: countryText.value },
    { label: '大洲', value: dash(i.continent) },
    { label: '省 / 州', value: dash(i.region) },
    { label: '城市', value: dash(i.city) },
    { label: '邮政编码', value: dash(i.postal) },
    { label: '经纬度', value: coordText.value },
    { label: '时区', value: timezoneText.value },
  ]
})

const netRows = computed(() => {
  const i = info.value
  if (!i) return []
  return [
    { label: '地址类型', value: active.value === 'ipv4' ? 'IPv4（常规地址，如 111.199.191.18）' : 'IPv6（新一代地址，由 8 段十六进制组成）' },
    { label: '运营商', value: dash(i.isp) },
    { label: '组织', value: dash(i.org) },
    { label: 'ASN', value: i.asn ? `AS${i.asn}` : '—' },
    { label: '数据来源', value: dash(i.source) },
  ]
})

onMounted(refresh)
</script>

<template>
  <div class="tool-body">
    <div class="ip-body">
      <section class="ip-hero">
        <div class="ip-hero-main">
          <div class="ip-tabs">
            <button
              class="ip-tab"
              :class="{ active: active === 'ipv4', off: !report?.ipv4 }"
              :disabled="!report?.ipv4"
              @click="switchTo('ipv4')"
            >
              IPv4
              <span v-if="report?.ipv4" class="ip-tab-dot" />
            </button>
            <button
              class="ip-tab"
              :class="{ active: active === 'ipv6', off: !report?.ipv6 }"
              :disabled="!report?.ipv6"
              @click="switchTo('ipv6')"
            >
              IPv6
              <span v-if="report?.ipv6" class="ip-tab-dot" />
            </button>
          </div>

          <div class="ip-value-row">
            <code class="ip-value" :class="{ 'ip-value-busy': busy, 'ip-value-long': active === 'ipv6' }">
              {{ info?.ip ?? (busy ? '查询中…' : missingHint || '—') }}
            </code>
            <span v-if="info" class="ip-badge">{{ active === 'ipv4' ? 'IPv4' : 'IPv6' }}</span>
          </div>
          <p class="ip-hero-sub">
            {{ locationText || (busy ? '' : missingHint || '暂无归属地信息') }}
          </p>
        </div>

        <div class="ip-hero-actions">
          <button class="pwd-btn primary" :disabled="busy" @click="refresh()">
            <Icon name="refresh" />
            <span>{{ busy ? '查询中…' : '刷新' }}</span>
          </button>
          <button class="pwd-btn" :disabled="!info" @click="copyIp()">
            <Icon :name="copied ? 'check' : 'copy'" />
            <span>{{ copied ? '已复制' : '复制IP' }}</span>
          </button>
        </div>
      </section>

      <p v-if="errorMsg" class="hash-error ip-error">{{ errorMsg }}</p>

      <div v-if="info" class="ip-grid">
        <section class="ip-card">
          <h2>地理位置</h2>
          <dl class="ip-meta">
            <div v-for="row in geoRows" :key="row.label" class="ip-row">
              <dt>{{ row.label }}</dt>
              <dd>{{ row.value }}</dd>
            </div>
          </dl>
        </section>

        <section class="ip-card">
          <h2>网络信息</h2>
          <dl class="ip-meta">
            <div v-for="row in netRows" :key="row.label" class="ip-row">
              <dt>{{ row.label }}</dt>
              <dd>{{ row.value }}</dd>
            </div>
          </dl>
        </section>
      </div>

      <div v-else-if="!busy && !errorMsg" class="ip-card ip-placeholder">
        点击「刷新」查询本机公网 IP
      </div>

      <section v-if="other" class="ip-card ip-other">
        <span class="ip-other-label">{{ active === 'ipv4' ? 'IPv6 地址' : 'IPv4 地址' }}</span>
        <code class="ip-other-value">{{ other.ip }}</code>
        <button class="ip-link" @click="switchTo(active === 'ipv4' ? 'ipv6' : 'ipv4')">切换查看</button>
      </section>
    </div>
  </div>
</template>
