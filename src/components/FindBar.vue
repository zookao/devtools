<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

/** 面板内查找栏：输入 + 计数 + 导航 + 大小写开关 + 关闭 */
const props = defineProps<{
  modelValue: string
  count: number
  index: number
  showNav: boolean
  caseSensitive: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [v: string]
  prev: []
  next: []
  close: []
  'update:caseSensitive': [v: boolean]
}>()

const inputEl = ref<HTMLInputElement | null>(null)

onMounted(() => {
  inputEl.value?.focus()
  inputEl.value?.select()
})

defineExpose({
  focus: () => {
    inputEl.value?.focus()
    inputEl.value?.select()
  },
})

const countText = computed(() => {
  if (!props.modelValue.trim()) return ''
  return props.count ? `${props.index + 1}/${props.count}` : '0'
})

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault()
    emit('close')
  } else if (e.key === 'Enter') {
    e.preventDefault()
    if (e.shiftKey) emit('prev')
    else emit('next')
  }
}
</script>

<template>
  <div class="findbar">
    <input
      ref="inputEl"
      class="find-input"
      type="text"
      placeholder="Find…"
      spellcheck="false"
      :value="modelValue"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      @keydown="onKeydown"
    />
    <span class="find-count">{{ countText }}</span>
    <button v-if="showNav" class="find-btn" title="Previous (⇧Enter)" :disabled="!count" @click="emit('prev')">↑</button>
    <button v-if="showNav" class="find-btn" title="Next (Enter)" :disabled="!count" @click="emit('next')">↓</button>
    <button
      class="find-btn"
      :class="{ on: caseSensitive }"
      title="Match case"
      @click="emit('update:caseSensitive', !caseSensitive)"
    >Aa</button>
    <button class="find-btn" title="Close (Esc)" @click="emit('close')">✕</button>
  </div>
</template>
