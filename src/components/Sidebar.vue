<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from './Icon.vue'
import { TOOLS } from '../tools/registry'
import { getTheme, toggleTheme } from '../theme'

const props = defineProps<{ activeId: string }>()
const emit = defineEmits<{ select: [id: string] }>()

const query = ref('')
const isDark = ref(getTheme() === 'dark')
const themeIcon = computed(() => (isDark.value ? 'sun' : 'moon'))

const groups = computed(() => {
  const q = query.value.trim().toLowerCase()
  const matched = q
    ? TOOLS.filter(
        (t) =>
          t.title.toLowerCase().includes(q) ||
          t.group.toLowerCase().includes(q) ||
          t.keywords.some((k) => k.toLowerCase().includes(q)),
      )
    : TOOLS
  // 保持注册顺序分组
  const order: string[] = []
  const map = new Map<string, typeof TOOLS>()
  for (const t of matched) {
    if (!map.has(t.group)) {
      map.set(t.group, [])
      order.push(t.group)
    }
    map.get(t.group)!.push(t)
  }
  return order.map((g) => ({ name: g, tools: map.get(g)! }))
})

function onToggleTheme() {
  toggleTheme()
  isDark.value = getTheme() === 'dark'
}
</script>

<template>
  <aside class="sidebar">
    <div class="search-box">
      <Icon name="search" />
      <input v-model="query" type="text" placeholder="Search tools…" spellcheck="false" />
    </div>

    <nav class="tool-list">
      <template v-if="groups.length">
        <div v-for="g in groups" :key="g.name">
          <div class="group-title">{{ g.name }}</div>
          <button
            v-for="t in g.tools"
            :key="t.id"
            class="tool-item"
            :class="{ active: t.id === props.activeId, planned: t.status === 'planned' }"
            @click="emit('select', t.id)"
          >
            <Icon :name="t.icon" />
            <span>{{ t.title }}</span>
            <span v-if="t.status === 'planned'" class="coming-soon">soon</span>
          </button>
        </div>
      </template>
      <div v-else class="no-results">No tools found.</div>
    </nav>

    <div class="sidebar-footer">
      <span>DevTools 0.1.0</span>
      <button class="icon-btn" title="Toggle theme" @click="onToggleTheme">
        <Icon :name="themeIcon" />
      </button>
    </div>
  </aside>
</template>
