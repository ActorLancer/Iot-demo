<script setup>
import { ref, onMounted } from "vue";
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
  CategoryScale
} from "chart.js";

ChartJS.register(
  Title,
  Tooltip,
  Legend,
  LineElement,
  PointElement,
  LinearScale,
  CategoryScale
);

const latest = ref(null);
const allData = ref([]);
const chartData = ref({
  labels: [],
  datasets: [
    {
      label: "温度 (°C)",
      borderColor: "red",
      data: []
    },
    {
      label: "湿度 (%)",
      borderColor: "blue",
      data: []
    }
  ]
});

// 定时获取最新数据
async function fetchData() {
  try {
    const latestResp = await axios.get("http://127.0.0.1:3000/data/latest");
    latest.value = latestResp.data;

    const allResp = await axios.get("http://127.0.0.1:3000/data/all");
    allData.value = allResp.data;

    chartData.value.labels = allData.value.map(d => d.created_at);
    chartData.value.datasets[0].data = allData.value.map(d => d.temperature);
    chartData.value.datasets[1].data = allData.value.map(d => d.humidity);
  } catch (err) {
    console.error("❌ API 请求失败", err);
  }
}

onMounted(() => {
  fetchData();
  setInterval(fetchData, 5000); // 每 5 秒刷新一次
});
</script>

<template>
  <div style="padding:20px">
    <h1>🌡️ IoT 仪表盘</h1>

    <div v-if="latest" style="margin:20px 0">
      <h2>最新数据</h2>
      <p>设备: {{ latest.device_id }}</p>
      <p>温度: {{ latest.temperature }} °C</p>
      <p>湿度: {{ latest.humidity }} %</p>
      <p>时间: {{ latest.created_at }}</p>
    </div>

    <h2>历史趋势</h2>
    <Line :data="chartData" />
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
</style>
