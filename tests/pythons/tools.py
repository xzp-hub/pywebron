from psutil import virtual_memory, swap_memory, cpu_times, disk_io_counters, net_io_counters
from asyncio import sleep as asyncio_sleep, gather
from datetime import datetime
import math


def cpu_task(n):
    print(f"[tasks.py] cpu_task{n} 计算开始")
    res = sum(i * i for i in range(20000000))
    print(f"[tasks.py] cpu_task 计算完成: {res}")
    return res


class SystemMonitoring:
    # IO 采样状态
    _disk_old = None
    _netw_old = None
    # 历史记录（用于图表）
    _io_history = {"disk_read": [], "disk_write": [], "net_read": [], "net_write": []}
    _max_history = 5  # 保留 5 个时间点

    @classmethod
    def rounder(cls, v):
        return round(v / 1024 ** 3, 2)

    @classmethod
    async def get_cpu_infos(cls, fast_mode: bool):
        time1 = cpu_times()
        await asyncio_sleep(0.1 if fast_mode else 1)
        time2 = cpu_times()
        time_sum1, time_sum2 = sum(time1), sum(time2)
        busy_time1 = time_sum1 - time1.idle
        busy_time2 = time_sum2 - time2.idle
        busy_time = busy_time2 - busy_time1
        full_time = time_sum2 - time_sum1
        usage = round((busy_time / full_time) * 100, 2)
        stats = f"{round(busy_time, 2)} S / {round(full_time, 2)} S"
        return {"usage": usage, "stats": stats}

    @classmethod
    async def get_ram_infos(cls):
        ram = virtual_memory()
        used, full = ram.used, ram.total
        usage = round((used / full) * 100, 2)
        stats = f"{cls.rounder(used)} GB / {cls.rounder(full)} GB"
        return {"usage": usage, "stats": stats}

    @classmethod
    async def get_vrm_infos(cls):
        vrm = swap_memory()
        used, total = vrm.used, vrm.total
        usage = round((used / total) * 100, 2)
        stats = f"{cls.rounder(used)} GB / {cls.rounder(total)} GB"
        return {"usage": usage, "stats": stats}

    @classmethod
    async def get_ios_infos(cls):
        disk_now, netw_now = disk_io_counters(), net_io_counters()

        # 计算速率（KB/s）
        disk_read_speed = (disk_now.read_bytes - cls._disk_old.read_bytes) / 1024
        disk_write_speed = (disk_now.write_bytes - cls._disk_old.write_bytes) / 1024
        net_read_speed = (netw_now.bytes_recv - cls._netw_old.bytes_recv) / 1024
        net_write_speed = (netw_now.bytes_sent - cls._netw_old.bytes_sent) / 1024

        # 累计（MB）
        disk_read_total = disk_now.read_bytes / (1024 ** 2)
        disk_write_total = disk_now.write_bytes / (1024 ** 2)
        net_read_total = netw_now.bytes_recv / (1024 ** 2)
        net_write_total = netw_now.bytes_sent / (1024 ** 2)

        # 更新基准值
        cls._disk_old, cls._netw_old = disk_now, netw_now

        # 记录历史数据
        cls._io_history["disk_read"].append(disk_read_speed)
        cls._io_history["disk_write"].append(disk_write_speed)
        cls._io_history["net_read"].append(net_read_speed)
        cls._io_history["net_write"].append(net_write_speed)

        # 保持 5 个时间点
        for key in cls._io_history:
            if len(cls._io_history[key]) > cls._max_history:
                cls._io_history[key].pop(0)

        # 计算 Y 轴刻度（自适应数值大小）
        def calc_y_ticks(read_values, write_values):
            all_vals = read_values + write_values
            max_val = max(all_vals) if all_vals else 0
            if max_val <= 0:
                return [100.0, 80.0, 60.0, 40.0, 20.0, 0.0]
            target_max = max_val * 1.2
            magnitude = 10 ** math.floor(math.log10(target_max)) if target_max >= 1 else 0.1
            ratio = target_max / magnitude
            if ratio <= 1:
                nice_max = magnitude
            elif ratio <= 2:
                nice_max = 2 * magnitude
            elif ratio <= 5:
                nice_max = 5 * magnitude
            else:
                nice_max = 10 * magnitude
            return [round(nice_max / 5 * i, 2) for i in range(5, -1, -1)]
        
        disk_y_ticks = calc_y_ticks(cls._io_history["disk_read"], cls._io_history["disk_write"])
        net_y_ticks = calc_y_ticks(cls._io_history["net_read"], cls._io_history["net_write"])

        return {
            "disk_io": {
                "read_speed": round(disk_read_speed, 2),
                "write_speed": round(disk_write_speed, 2),
                "read_total": round(disk_read_total, 2),
                "write_total": round(disk_write_total, 2)
            },
            "net_io": {
                "read_speed": round(net_read_speed, 2),
                "write_speed": round(net_write_speed, 2),
                "read_total": round(net_read_total, 2),
                "write_total": round(net_write_total, 2)
            },
            "y_ticks": {
                "disk": disk_y_ticks,
                "net": net_y_ticks
            }
        }

    @classmethod
    async def run(cls, fast_mode: bool = False):
        t0 = datetime.now()
        hms = t0.strftime("%H:%M:%S")
        need_io_baseline = cls._disk_old is None or cls._netw_old is None
        if need_io_baseline:
            cls._disk_old = disk_io_counters()
            cls._netw_old = net_io_counters()

        cpu, ram, vrm = await gather(
            cls.get_cpu_infos(fast_mode),
            cls.get_ram_infos(),
            cls.get_vrm_infos(),
        )

        placeholder_ios = {
            "disk_io": {"read_speed": 0.0, "write_speed": 0.0,
                        "read_total": cls._disk_old.read_bytes / (1024 ** 2),
                        "write_total": cls._disk_old.write_bytes / (1024 ** 2)},
            "net_io": {"read_speed": 0.0, "write_speed": 0.0,
                       "read_total": cls._netw_old.bytes_recv / (1024 ** 2),
                       "write_total": cls._netw_old.bytes_sent / (1024 ** 2)},
            "y_ticks": {"disk": [100.0, 80.0, 60.0, 40.0, 20.0, 0.0],
                        "net": [100.0, 80.0, 60.0, 40.0, 20.0, 0.0]}
        }

        if need_io_baseline:
            return {'time': hms, 'info': {"cpu": cpu, "ram": ram, "vrm": vrm, "ios": placeholder_ios}}

        ios = await cls.get_ios_infos()
        return {'time': hms, 'info': {"cpu": cpu, "ram": ram, "vrm": vrm, "ios": ios}}
