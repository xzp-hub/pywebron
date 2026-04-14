import { ref } from 'vue'

// 直接读取系统主题
const currentTheme = ref(
    window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
)

// 监听系统主题变化，自动跟随
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
    currentTheme.value = e.matches ? 'dark' : 'light'
    applyTheme()
})

function applyTheme() {
    const html = document.documentElement
    if (currentTheme.value === 'dark') {
        html.setAttribute('data-theme', 'dark')
        html.setAttribute('theme-mode', 'dark')
    } else {
        html.removeAttribute('data-theme')
        html.removeAttribute('theme-mode')
    }
}

export function useTheme() {
    const setTheme = (theme) => {
        currentTheme.value = theme === 'dark' ? 'dark' : 'light'
        applyTheme()
    }

    const toggleTheme = () => {
        setTheme(currentTheme.value === 'light' ? 'dark' : 'light')
    }

    const initTheme = () => {
        applyTheme()
    }

    return {
        currentTheme,
        setTheme,
        toggleTheme,
        initTheme
    }
}
