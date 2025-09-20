<script setup>
import { ref, onMounted, computed, watch } from "vue";
import axios from "axios";
import { Line } from "vue-chartjs";
import {
    Chart as ChartJS,
    Title,
    Tooltip,
    Legend,
    LineElement,
    PointElement,
    LinearScale,
    CategoryScale,
} from "chart.js";

ChartJS.register(
    Title,
    Tooltip,
    Legend,
    LineElement,
    PointElement,
    LinearScale,
    CategoryScale,
);

const latest = ref(null);
const allData = ref([]);
const chartKey = ref(0); // 用于强制重新渲染图表

// 使用计算属性确保响应性
// 使用 compute() 代替 ref() 来创建 chartData，确保数据变化时自动重新计算
const chartData = computed(() => {
    if (!allData.value || allData.value.length === 0) {
        return {
            labels: [],
            datasets: [
                {
                    label: "温度 (°C)",
                    borderColor: "#ff6384",
                    backgroundColor: "rgba(255, 99, 132, 0.1)",
                    data: [],
                    tension: 0.1,
                },
                {
                    label: "湿度 (%)",
                    borderColor: "#36a2eb",
                    backgroundColor: "rgba(54, 162, 235, 0.1)",
                    data: [],
                    tension: 0.1,
                },
            ],
        };
    }

    return {
        labels: allData.value.map((d) => {
            // 格式化时间显示
            const date = new Date(d.created_at);
            return date.toLocaleTimeString("zh-CN", {
                hour: "2-digit",
                minute: "2-digit",
                second: "2-digit",
            });
        }),
        datasets: [
            {
                label: "温度 (°C)",
                borderColor: "#ff6384",
                backgroundColor: "rgba(255, 99, 132, 0.1)",
                data: allData.value.map((d) => parseFloat(d.temperature)),
                tension: 0.1,
                fill: false,
            },
            {
                label: "湿度 (%)",
                borderColor: "#36a2eb",
                backgroundColor: "rgba(54, 162, 235, 0.1)",
                data: allData.value.map((d) => parseFloat(d.humidity)),
                tension: 0.1,
                fill: false,
            },
        ],
    };
});

// 图表配置选项
// 设置相应式布局和固定高度
// 优化颜色和样式
// 禁用动画以提高性能
const chartOptions = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
        title: {
            display: true,
            text: "温湿度历史趋势图",
        },
        legend: {
            display: true,
            position: "top",
        },
    },
    scales: {
        x: {
            display: true,
            title: {
                display: true,
                text: "时间",
            },
        },
        y: {
            display: true,
            title: {
                display: true,
                text: "数值",
            },
        },
    },
    animation: {
        duration: 0, // 禁用动画以提高性能
    },
};

// 定时获取最新数据
async function fetchData() {
    try {
        console.log("🔄 正在获取数据...");

        const latestResp = await axios.get("http://127.0.0.1:3000/data/latest");
        latest.value = latestResp.data;
        console.log("📊 最新数据:", latest.value);

        const allResp = await axios.get("http://127.0.0.1:3000/data/all");
        allData.value = allResp.data;
        console.log("📈 历史数据数量:", allData.value.length);

        // 强制更新图表
        chartKey.value++;
    } catch (err) {
        console.error("❌ API 请求失败", err.message);
        console.error("🔍 请确认后端服务是否正常运行在 http://127.0.0.1:3000");
    }
}

// 监听数据变化
watch(
    allData,
    (newData) => {
        console.log("🔄 数据已更新，条目数:", newData.length);
    },
    { deep: true },
);

onMounted(() => {
    console.log("🚀 组件已挂载，开始获取数据");
    fetchData();
    setInterval(fetchData, 5000); // 每 5 秒刷新一次
});
</script>

<template>
    <div style="padding: 20px">
        <h1>🌡️ IoT 仪表盘</h1>

        <div
            v-if="latest"
            style="
                margin: 20px 0;
                padding: 15px;
                border: 1px solid #ddd;
                border-radius: 5px;
            "
        >
            <h2>最新数据</h2>
            <p><strong>设备:</strong> {{ latest.device_id }}</p>
            <p><strong>温度:</strong> {{ latest.temperature }} °C</p>
            <p><strong>湿度:</strong> {{ latest.humidity }} %</p>
            <p><strong>时间:</strong> {{ latest.created_at }}</p>
        </div>

        <div
            v-else
            style="
                margin: 20px 0;
                padding: 15px;
                background-color: #fff3cd;
                border: 1px solid #ffeaa7;
                border-radius: 5px;
            "
        >
            <p>⏳ 正在加载最新数据...</p>
        </div>

        <div style="margin: 20px 0">
            <h2>历史趋势 (数据点: {{ allData.length }})</h2>
            <div style="height: 400px; position: relative">
                <Line
                    :key="chartKey"
                    :data="chartData"
                    :options="chartOptions"
                />
            </div>
        </div>

        <div
            v-if="allData.length === 0"
            style="
                margin: 20px 0;
                padding: 15px;
                background-color: #f8d7da;
                border: 1px solid #f5c6cb;
                border-radius: 5px;
            "
        >
            <p>⚠️ 暂无历史数据，请确认后端数据服务正常</p>
        </div>
    </div>
</template>

<style scoped>
.logo {
    height: 6em;
    padding: 1.5em;
    will-change: filter;
    transition: filter 300ms;
}
.logo:hover {
    filter: drop-shadow(0 0 2em #646cffaa);
}
.logo.vue:hover {
    filter: drop-shadow(0 0 2em #42b883aa);
}

h1 {
    color: #2c3e50;
    margin-bottom: 30px;
}

h2 {
    color: #34495e;
    margin-bottom: 15px;
}

p {
    margin: 8px 0;
    line-height: 1.5;
}

strong {
    font-weight: 600;
}
</style>
