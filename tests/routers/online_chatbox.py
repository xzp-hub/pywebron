from asyncio import sleep as asyncio_sleep
from traceback import format_exc

from pywebron import Router, Worker, App, StreamSendModes
from tests.utills.tools import cpu_task

router = Router(title="聊天室")
stream = router.stream


class ChatRoomStruct(stream.struct):
    n: int = 42


@stream.handle("chat_room_stream")
async def chat_room(server: stream.server, worker: Worker, struct: ChatRoomStruct):
    try:
        await server.send(200, "欢迎加入聊天室", {"type": "system"}, save_history=True)
        while True:
            match res := await server.recv():
                case None | {}:
                    await asyncio_sleep(0.1)
                case "multicast_test":
                    (wids := list(App.get_windows().keys())).remove(server.window_id)
                    await server.send(
                        200,
                        "组播功能测试",
                        {"type": "chat"},
                        send_mode=StreamSendModes.MULTICAST,
                        mcast_win_ids=wids[0:1],
                    )
                case "worker_test":
                    res = await worker.run(cpu_task, n := struct.n)
                    await server.send(
                        200,
                        f"Worker 任务完成，n: {n}, result: {res}",
                        {"type": "chat", "result": res, "n": n},
                        send_mode=StreamSendModes.UNITYCAST,
                    )
                case _:
                    if res:
                        await server.send(
                            200,
                            f"收到 {res}",
                            {"type": "chat"},
                            send_mode=StreamSendModes.UNITYCAST,
                        )
    except Exception:
        await server.send(500, "聊天室错误", format_exc())
