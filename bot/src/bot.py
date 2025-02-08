from typing import Any, Dict, Optional, Sequence, Union
import os
from aiogram import Bot, Dispatcher, Router, F
from aiogram.types import Message, ChatMemberUpdated, ChatMemberUpdated
from aiogram.filters import Command, BaseFilter, ChatMemberUpdatedFilter, IS_NOT_MEMBER, MEMBER, ADMINISTRATOR
from dotenv import load_dotenv
from db import init_database
from telethon import TelegramClient, events
from telethon.tl.types import PeerUser, PeerChat, PeerChannel
from telethon.tl.functions.channels import GetParticipantRequest
from telethon.tl.types import ChannelParticipantsSearch

load_dotenv()

init_database()

API_ID = os.getenv("API_ID")
API_HASH = os.getenv("API_HASH")

client = TelegramClient("session_name", API_ID, API_HASH)
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

@router.my_chat_member(ChatMemberUpdatedFilter(member_status_changed=IS_NOT_MEMBER >> MEMBER))
async def bot_added_to_channel(event: ChatMemberUpdated):
    chat_info = await bot.get_chat(event.chat.id)
    if chat_info.permissions and chat_info.permissions.can_send_messages:
        await event.answer(
            text=f"Привет! Спасибо, что добавили меня в "
                 f"как обычного участника. ID чата: {event.chat.id}"
        )
    else:
        print("Как-нибудь логируем эту ситуацию")

@router.my_chat_member(
    ChatMemberUpdatedFilter(
        member_status_changed=IS_NOT_MEMBER >> ADMINISTRATOR
    )
)
async def bot_added_as_admin(event: ChatMemberUpdated):
    chat_info = await bot.get_chat(event.chat.id)
    # Самый простой случай: бот добавлен как админ.
    # Легко можем отправить сообщение
    if chat_info.permissions and not chat_info.permissions.can_invite_users:
        await event.answer(
            text=f"Привет! Спасибо, что добавили меня в "
                 f"как обычного участника. ID чата: {event.chat.id}"
        )
    else:
        print("Как-нибудь логируем эту ситуацию")
    await event.answer(
        text=f"Привет! Спасибо, что добавили меня"
             f"как администратора. ID чата: {event.chat.id}"
    )

async def remove_user(chat_id:int, user_id:int):
    user = await client(GetParticipantRequest(chat_id, 352892531))
    entity = await client.get_entity(PeerUser(user_id))
    msg = await client.kick_participant(chat_id, entity)
    if hasattr(msg, 'delete'):
        await msg.delete()
    # await bot.ban_chat_member(chat_id, user_id)
    # await bot.unban_chat_member(chat_id, user_id)


@router.channel_post(CommandFilter('ban'))
async def ban_user(message: Message):
    try:
        if not message.text or len(message.text.split()) < 2:
            await message.answer("Пожалуйста, укажите пользователя после команды /ban.")
            return
        chat_id = message.chat.id
        # users = await client.get_participants(await client.get_input_entity(chat_id))
        user_to_ban = await bot.get_chat_member(chat_id, 352892531)
        
        if not user_to_ban:
            await message.answer("Пользователь не найден.")
            return

        await remove_user(chat_id, user_to_ban.user.id)
        await message.answer(f"Пользователь {user_to_ban.user.full_name} был забанен.")
    except Exception as e:
        await message.answer(f"Не удалось забанить пользователя. Ошибка: {e}")



async def main() -> None:
    dp.include_router(router)
    await client.start()
    await dp.start_polling(bot)


if __name__ == '__main__':
    import asyncio
    asyncio.run(main())