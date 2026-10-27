<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../components/Icon.vue'
import { clipboardWriteText, hashMd5 } from '../../api'

const SAMPLE = `The quick brown fox jumps over the lazy dog`

const input = ref('')
const result = ref('')
const computing = ref(false)
const copied = ref(false)
const errorMsg = ref('')

const canCompute = computed(() => input.value.trim() !== '')

async function compute() {
  if (!canCompute.value || computing.value) return
  computing.value = true
  errorMsg.value = ''
  try {
    result.value = await hashMd5(input.value)
  } catch (e) {
    result.value = ''
    errorMsg.value = String(e)
  } finally {
    computing.value = false
  }
}

function loadSample() {
  input.value = SAMPLE
  result.value = ''
  errorMsg.value = ''
}

function clearAll() {
  input.value = ''
  result.value = ''
  errorMsg.value = ''
}

async function copyResult() {
  if (!result.value) return
  await clipboardWriteText(result.value)
  copied.value = true
  setTimeout(() => (copied.value = false), 1500)
}
</script>

<template>
  <div class="tool-body">
    <div class="hash-body">
      <p class="hash-desc">一种广泛使用的哈希函数，产生128位（16字节）哈希值。不可逆，不适用于安全场景。</p>

      <div class="hash-actions">
        <button class="icon-btn" @click="loadSample()">
          <Icon name="flask" />
          <span>加载示例</span>
        </button>
        <button class="icon-btn" @click="clearAll()">
          <Icon name="x" />
          <span>清空</span>
        </button>
      </div>

      <div class="hash-split">
        <section class="hash-col">
          <div class="hash-col-head">
            <span class="hash-col-label">原始文本</span>
          </div>
          <textarea
            v-model="input"
            class="hash-textarea"
            spellcheck="false"
            placeholder="请输入需要处理的文本..."
          />
        </section>

        <section class="hash-col">
          <div class="hash-col-head">
            <span class="hash-col-label">哈希结果</span>
            <button
              class="ts-icon-btn"
              :title="copied ? 'Copied!' : 'Copy hash'"
              :disabled="!result"
              @click="copyResult()"
            >
              <Icon :name="copied ? 'check' : 'copy'" />
            </button>
          </div>
          <textarea
            class="hash-textarea readonly"
            readonly
            spellcheck="false"
            :value="result"
            placeholder="结果将显示在这里..."
          />
        </section>
      </div>

      <div>
        <button class="hash-compute" :disabled="!canCompute || computing" @click="compute()">
          <Icon name="lock" />
          <span>{{ computing ? '计算中…' : '计算' }}</span>
        </button>
      </div>

      <p v-if="errorMsg" class="hash-error">{{ errorMsg }}</p>

      <section class="hash-note">
        <h2>算法说明</h2>
        <p>MD5: 一种广泛使用的哈希函数，产生128位（16字节）哈希值。不可逆，不适用于安全场景。</p>
        <p>MD5可以用于文件校验，但因已被证明存在碰撞攻击，不应用于安全场景。</p>
      </section>
    </div>
  </div>
</template>
