import { ref, watch, onMounted } from 'vue'

const THEME_KEY = 'pywebron-theme'
const currentTheme = ref('light')

export function useTheme() {
    const setTheme = (theme) => {
        const validTheme = theme === 'dark' ? 'dark' : 'light'
        currentTheme.value = validTheme

        // 更新 HTML data-theme 属性
        document.documentElement.setAttribute('data-theme', validTheme)

        // 保存到 localStorage
        try {
            localStorage.setItem(THEME_KEY, validTheme)
        } catch (e) {
            console.warn('无法保存主题设置:', e)
        }
    }

    const toggleTheme = () => {
        setTheme(currentTheme.value === 'light' ? 'dark' : 'light')
    }

    const initTheme = () => {
        // 从 localStorage 读取保存的主题
        let savedTheme = null
        try {
            savedTheme = localStorage.getItem(THEME_KEY)
        } catch (e) {
            console.warn('无法读取主题设置:', e)
        }

        // 如果没有保存的主题，检测系统偏好
        if (!savedTheme) {
            const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
            savedTheme = prefersDark ? 'dark' : 'light'
        }

        setTheme(savedTheme)

        // 监听系统主题变化
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
        const handleChange = (e) => {
            // 只在没有用户手动设置时才跟随系统
            try {
                const userTheme = localStorage.getItem(THEME_KEY)
                if (!userTheme) {
                    setTheme(e.matches ? 'dark' : 'light')
                }
            } catch (err) {
                console.warn('主题切换失败:', err)
            }
        }

        if (mediaQuery.addEventListener) {
            mediaQuery.addEventListener('change', handleChange)
        } else {
            // 兼容旧浏览器
            mediaQuery.addListener(handleChange)
        }
    }

    return {
        currentTheme,
        setTheme,
        toggleTheme,
        initTheme
    }
}
