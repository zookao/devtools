<script setup lang="ts">
import { onMounted, reactive, ref, watch } from 'vue'
import Icon from '../../components/Icon.vue'
import { clipboardWriteText, uuidGenerate, type UuidOptions } from '../../api'

const STORAGE_KEY = 'devtools.uuid.options'

const DEFAULTS: UuidOptions = {
  count: 5,
  uppercase: false,
  hyphens: true,
  braces: false,
}

function loadOptions(): UuidOptions {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) return { ...DEFAULTS, ...JSON.parse(raw) }
  } catch {
    /* 忽略损坏的本地缓存 */
  }
  return { ...DEFAULTS }
}

const options = reactive<UuidOptions>(loadOptions())

const rows = ref<string[]>([])
const errorMsg = ref('')
const busy = ref(false)
const copiedIndex = ref(-1)
const allCopied = ref(false)

watch(
  options,
  (v) => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(v))
    } catch {
      /* 忽略写入失败 */
    }
  },
  { deep: true },
)

async function generate() {
  if (busy.value) return
  busy.value = true
  errorMsg.value = ''
  try {
    rows.value = await uuidGenerate({ ...options })
  } catch (e) {
    rows.value = []
    errorMsg.value = String(e)
  } finally {
    busy.value = false
  }
}

async function copyOne(i: number) {
  await clipboardWriteText(rows.value[i])
  copiedIndex.value = i
  setTimeout(() => {
    if (copiedIndex.value === i) copiedIndex.value = -1
  }, 1500)
}

async function copyAll() {
  if (!rows.value.length) return
  await clipboardWriteText(rows.value.join('\n'))
  allCopied.value = true
  setTimeout(() => (allCopied.value = false), 1500)
}

function resetOptions() {
  Object.assign(options, DEFAULTS)
  rows.value = []
  errorMsg.value = ''
}

onMounted(generate)
</script>

<template>
  <div class="tool-body">
    <div class="pwd-body">
      <section class="pwd-card">
        <div class="pwd-grid">
          <!-- 左列 -->
          <div class="pwd-col">
            <label class="pwd-label" for="uuid-count">生成数量</label>
            <input
              id="uuid-count"
              v-model.number="options.count"
              class="pwd-input"
              type="number"
              min="1"
              max="100"
            />
            <p class="pwd-hint">范围 1 - 100，批量生成时每行一个</p>

            <div class="pwd-checks">
              <label class="pwd-check">
                <input v-model="options.uppercase" type="checkbox" />
                <span>大写字母</span>
              </label>
              <label class="pwd-check">
                <input v-model="options.hyphens" type="checkbox" />
                <span>包含连字符（8-4-4-4-12）</span>
              </label>
              <label class="pwd-check">
                <input v-model="options.braces" type="checkbox" />
                <span>两端加花括号 { }</span>
              </label>
            </div>
          </div>

          <!-- 右列 -->
          <div class="pwd-col">
            <label class="pwd-label">格式说明</label>
            <p class="pwd-hint uuid-hint">生成 RFC 4122 版本 4（随机）UUID，共 128 位。</p>
            <div class="uuid-sample">
              <span class="uuid-sample-label">示例</span>
              <code>{{ rows[0] || '——' }}</code>
            </div>
            <p class="pwd-hint">版本号固定为 4，变体位为 RFC 4122（8/9/a/b）</p>
          </div>
        </div>

        <div class="pwd-actions">
          <button class="pwd-btn primary" :disabled="busy" @click="generate()">
            <Icon name="refresh" />
            <span>{{ busy ? '生成中…' : '生成' }}</span>
          </button>
          <button class="pwd-btn" :disabled="!rows.length" @click="copyAll()">
            <Icon :name="allCopied ? 'check' : 'copy'" />
            <span>{{ allCopied ? '已复制' : '复制全部' }}</span>
          </button>
          <button class="pwd-btn" @click="resetOptions()">
            <Icon name="trash" />
            <span>重置</span>
          </button>
        </div>

        <p v-if="errorMsg" class="hash-error">{{ errorMsg }}</p>
      </section>

      <section class="pwd-results">
        <h2>生成结果</h2>
        <div v-if="rows.length" class="pwd-list">
          <div v-for="(u, i) in rows" :key="i" class="pwd-row">
            <code>{{ u }}</code>
            <button
              class="ts-icon-btn"
              :title="copiedIndex === i ? 'Copied!' : 'Copy'"
              @click="copyOne(i)"
            >
              <Icon :name="copiedIndex === i ? 'check' : 'copy'" />
            </button>
          </div>
        </div>
        <p v-else class="pwd-empty">点击「生成」按钮创建 UUID</p>
      </section>
    </div>
  </div>
</template>
