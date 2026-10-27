<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import Icon from '../../components/Icon.vue'
import { clipboardWriteText, passwordGenerate, type PasswordOptions } from '../../api'

const STORAGE_KEY = 'devtools.password.options'

const DEFAULTS: PasswordOptions = {
  length: 12,
  count: 10,
  upper: true,
  lower: true,
  digits: true,
  symbols: true,
  requireEach: true,
  extraChars: '',
  excludeChars: '',
}

function loadOptions(): PasswordOptions {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) return { ...DEFAULTS, ...JSON.parse(raw) }
  } catch {
    /* 忽略损坏的本地缓存 */
  }
  return { ...DEFAULTS }
}

const options = reactive<PasswordOptions>(loadOptions())

/** 相似字符集合（忽略相似字符开关使用） */
const SIMILAR = 'Il1O0o'

const ignoreSimilar = ref(false)
const ignoreAmbiguous = ref(false)

const AMBIGUOUS = '{}[]()/\\\'"`~,;:.<>'

const rows = ref<string[]>([])
const errorMsg = ref('')
const busy = ref(false)
const copiedIndex = ref(-1)
const allCopied = ref(false)

const effectiveOptions = computed<PasswordOptions>(() => {
  let exclude = options.excludeChars
  if (ignoreSimilar.value) {
    for (const c of SIMILAR) if (!exclude.includes(c)) exclude += c
  }
  if (ignoreAmbiguous.value) {
    for (const c of AMBIGUOUS) if (!exclude.includes(c)) exclude += c
  }
  return { ...options, excludeChars: exclude }
})

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
    rows.value = await passwordGenerate(effectiveOptions.value)
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
  ignoreSimilar.value = false
  ignoreAmbiguous.value = false
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
            <label class="pwd-label" for="pwd-length">密码长度</label>
            <input id="pwd-length" v-model.number="options.length" class="pwd-input" type="number" min="4" max="128" />
            <p class="pwd-hint">范围 4 - 128，推荐 12+ 保证强度</p>

            <div class="pwd-checks">
              <label class="pwd-check">
                <input v-model="options.upper" type="checkbox" />
                <span>包含大写字母 A-Z</span>
              </label>
              <label class="pwd-check">
                <input v-model="options.lower" type="checkbox" />
                <span>包含小写字母 a-z</span>
              </label>
              <label class="pwd-check">
                <input v-model="options.digits" type="checkbox" />
                <span>包含数字 0-9</span>
              </label>
              <label class="pwd-check">
                <input v-model="options.symbols" type="checkbox" />
                <span>包含符号</span>
              </label>
              <label class="pwd-check">
                <input v-model="options.requireEach" type="checkbox" />
                <span>强制每个密码至少包含所选类型</span>
              </label>
            </div>
          </div>

          <!-- 右列 -->
          <div class="pwd-col">
            <label class="pwd-label" for="pwd-count">生成数量</label>
            <input id="pwd-count" v-model.number="options.count" class="pwd-input" type="number" min="1" max="100" />
            <p class="pwd-hint">范围 1 - 100</p>

            <label class="pwd-label" for="pwd-extra">自定义追加字符</label>
            <input
              id="pwd-extra"
              v-model="options.extraChars"
              class="pwd-input"
              type="text"
              placeholder="例如：€¥√"
              spellcheck="false"
            />
            <p class="pwd-hint">这些字符会被加入候选集合</p>

            <label class="pwd-label" for="pwd-exclude">排除字符</label>
            <input
              id="pwd-exclude"
              v-model="options.excludeChars"
              class="pwd-input"
              type="text"
              placeholder="例如：Il1O0o"
              spellcheck="false"
            />

            <div class="pwd-switches">
              <button
                class="pwd-switch"
                :class="{ on: ignoreSimilar }"
                @click="ignoreSimilar = !ignoreSimilar"
              >
                忽略相似字符({{ SIMILAR }}) : {{ ignoreSimilar ? '开' : '关' }}
              </button>
              <button
                class="pwd-switch"
                :class="{ on: ignoreAmbiguous }"
                @click="ignoreAmbiguous = !ignoreAmbiguous"
              >
                忽略歧义符号({{ AMBIGUOUS }}) : {{ ignoreAmbiguous ? '开' : '关' }}
              </button>
            </div>
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
          <div v-for="(p, i) in rows" :key="i" class="pwd-row">
            <code>{{ p }}</code>
            <button
              class="ts-icon-btn"
              :title="copiedIndex === i ? 'Copied!' : 'Copy'"
              @click="copyOne(i)"
            >
              <Icon :name="copiedIndex === i ? 'check' : 'copy'" />
            </button>
          </div>
        </div>
        <p v-else class="pwd-empty">点击「生成」按钮创建密码</p>
      </section>
    </div>
  </div>
</template>
