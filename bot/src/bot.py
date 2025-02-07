from typing import Any, Dict, Optional, Sequence, Union
import os
from aiogram import Bot, Dispatcher, Router, F
from aiogram.types import Message
from aiogram.filters import Command, BaseFilter
from dotenv import load_dotenv
from db import init_database

load_dotenv()

init_database()

bot: Bot = Bot(token=os.getenv('BOT_TOKEN', ''))
dp: Dispatcher = Dispatcher()

class CommandFilter(BaseFilter):

    def __init__(self, command: str):
        self.command = command
    """Фильтр для /start с возможным упоминанием бота"""
    async def __call__(self, message: Message) -> bool:
        bot_username = (await bot.get_me()).username
        if not message.text:
            return False
        
        text = message.text.lower().strip()
        if text.startswith(f"@{bot_username} /{self.command}".lower()):
            text = f"/{self.command}"
        return text == f"/{self.command}"

async def bot_name() -> None | str:
    return (await bot.get_me()).username

# router for chat messages
router = Router()
router.message.filter(F.chat.type != "private")

def is_direct_message(message: Message) -> bool:
    return message.chat.type == 'private'
# Handler for /start command in private chats
@dp.message(Command('start'), F.chat.type == "private")
async def start_private(message: Message) -> None:
    await message.answer('Привет! Это личное сообщение. Бот запущен в приватном чате.')

# Handler for /start command in public chats (groups or supergroups)
@router.message(CommandFilter('start'))
async def start_public(message: Message) -> None:
    await message.answer('Привет! Это публичный чат. Бот запущен в группе.')

async def main() -> None:
    dp.include_router(router)
    await dp.start_polling(bot)

if __name__ == '__main__':
    import asyncio
    asyncio.run(main())