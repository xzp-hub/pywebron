from asyncio import sleep as asyncio_sleep
from traceback import format_exc

from pywebron import Router, Worker, App, StreamSendModes
from tools import cpu_task

router = Router()


class ChatRoomStruct(router.stream.struct):
    n: int = 42


@router.stream.handle("chat_room_stream")
async def chat_room(stream: router.stream, worker: Worker, struct: ChatRoomStruct):
    try:
        await stream.send(200, "欢迎加入聊天室", {"type": "system"})
        while True:
            match res := await stream.recv():
                case None | {}:
                    await asyncio_sleep(0.1)
                case "multicast_test":
                    (wids := list(App.get_windows().keys())).remove(stream.window_id)
                    await stream.send(
                        200,
                        "组播功能测试",
                        {"type": "chat"},
                        send_mode=StreamSendModes.MULTICAST,
                        mcast_win_ids=wids[0:1],
                    )
                case "worker_test":
                    res = await worker.run(cpu_task, n := struct.n)
                    await stream.send(
                        200,
                        f"Worker 任务完成，n: {n}, result: {res}",
                        {"type": "chat", "result": res, "n": n},
                        send_mode=StreamSendModes.UNITYCAST,
                    )
                case _:
                    if res:
                        await stream.send(
                            200,
                            f"收到 {res}",
                            {"type": "chat"},
                            send_mode=StreamSendModes.UNITYCAST,
                        )
    except Exception:
        await stream.send(500, "聊天室错误", format_exc())


def create_chat_router():
    """聊天室分组"""
    return router
