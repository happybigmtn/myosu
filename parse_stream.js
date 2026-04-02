#!/usr/bin/env node
// Parse Codex JSONL output from `codex exec --json` for readable terminal display.

const readline = require('readline');

const colors = {
  reset: '\x1b[0m',
  dim: '\x1b[2m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
  red: '\x1b[31m',
  gray: '\x1b[90m',
};

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false,
});

let toolCount = 0;

function printBlock(prefix, text, color = colors.gray, limit = 6) {
  if (!text) return;
  const lines = String(text)
    .split('\n')
    .map((line) => line.trimEnd())
    .filter((line) => line.trim().length > 0);
  if (lines.length === 0) return;
  const shown = lines.slice(0, limit);
  for (const line of shown) {
    const clipped = line.length > 140 ? `${line.slice(0, 137)}...` : line;
    console.log(`${color}${prefix}${clipped}${colors.reset}`);
  }
  if (lines.length > limit) {
    console.log(`${colors.dim}${prefix}... +${lines.length - limit} more lines${colors.reset}`);
  }
}

rl.on('line', (line) => {
  const trimmed = line.trim();
  if (!trimmed) return;

  let data;
  try {
    data = JSON.parse(trimmed);
  } catch (_err) {
    process.stderr.write(`${trimmed}\n`);
    return;
  }

  switch (data.type) {
    case 'thread.started':
      console.log(`${colors.yellow}● Thread ${data.thread_id}${colors.reset}`);
      break;
    case 'turn.started':
      console.log(`${colors.yellow}● Turn started${colors.reset}`);
      break;
    case 'item.started': {
      const item = data.item || {};
      if (item.type === 'command_execution') {
        toolCount += 1;
        console.log(`\n${colors.cyan}🔧 command${colors.reset}`);
        printBlock('   ', item.command, colors.dim, 2);
      }
      break;
    }
    case 'item.completed': {
      const item = data.item || {};
      if (item.type === 'reasoning') {
        printBlock(`${colors.dim}thinking: `, item.text, colors.dim, 3);
      } else if (item.type === 'command_execution') {
        const exitCode = item.exit_code;
        const color = exitCode === 0 ? colors.green : colors.red;
        const label = exitCode === 0 ? '   ↳ result: ' : `   ↳ exit ${exitCode}: `;
        printBlock(label, item.aggregated_output || '(no output)', color);
      } else if (item.type === 'agent_message') {
        const text = item.text || '';
        if (text.trim()) {
          process.stdout.write(`\n${text}\n`);
        }
      } else {
        printBlock(`${colors.dim}${item.type || 'item'}: `, item.text || JSON.stringify(item), colors.dim, 3);
      }
      break;
    }
    case 'turn.completed': {
      const usage = data.usage || {};
      const input = usage.input_tokens || 0;
      const output = usage.output_tokens || 0;
      const cached = usage.cached_input_tokens || 0;
      console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
      console.log(
        `${colors.green}✅ Done${colors.reset} | Tokens: ↓${input.toLocaleString()} ↑${output.toLocaleString()} | Cached: ${cached.toLocaleString()} | Tools: ${toolCount}`
      );
      break;
    }
    case 'error':
      console.log(`${colors.red}❌ ${data.message || JSON.stringify(data)}${colors.reset}`);
      break;
    default:
      break;
  }
});

