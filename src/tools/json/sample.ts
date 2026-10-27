/** 示例数据：刻意包含注释与尾随逗号，展示容错能力 */
export const SAMPLE = `{
  // DevTools sample — comments & trailing commas are tolerated
  "store": {
    "book": [
      { "category": "reference", "author": "Nigel Rees", "title": "Sayings of the Century", "price": 8.95, },
      { "category": "fiction", "author": "Evelyn Waugh", "title": "Sword of Honour", "price": 12.99, },
    ],
    "bicycle": { "color": "red", "price": 19.95 },
  },
  "expensive": 10
}`
