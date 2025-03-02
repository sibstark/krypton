import os
from aiogram import Bot, Dispatcher, Router, F
from aiogram.types import Message, ChatMemberUpdated
from aiogram.filters import Command, BaseFilter, ChatMemberUpdatedFilter, IS_NOT_MEMBER, MEMBER, ADMINISTRATOR
from dotenv import load_dotenv
from sqlalchemy import select
from telethon import TelegramClient
from telethon.tl.types import PeerUser
from telethon.tl.functions.channels import GetParticipantRequest
from db import User, Channel, db_config, db_initializer
from datetime import datetime, timezone

load_dotenv()


API_ID = os.getenv("API_ID")
API_HASH = os.getenv("API_HASH")

client = TelegramClient("session_name", API_ID, API_HASH)
bot: Bot = Bot(token=os.getenv('BOT_TOKEN', ''))
dp: Dispatcher = Dispatcher()

# возможно нужно раделить на 2 метода - single responsibility
async def update_channel_and_owner(chat_id: int) -> None:
    chat_info = await bot.get_chat(chat_id)
    admins = await chat_info.get_administrators()

    # Find the owner of the chat
    owner = next((admin for admin in admins if admin.status == 'creator'), None)
    if owner:
        # Database connection
        async with db_config.async_session() as session:
            user = await session.execute(
                select(User).filter_by(telegram_id=owner.user.id)
            )
            user_exists = user.scalar()
                # Check if the user already exists in the database
            if not user_exists:
                # Add the owner to the database
                new_user = User(
                    telegram_id=owner.user.id,
                    username=owner.user.username,
                    first_name=owner.user.first_name,
                    last_name=owner.user.last_name
                )
                session.add(new_user)
                print(f"User {owner.user.username} added to the database.")
            else:
                # Update the existing user's information
                user_exists.username = owner.user.username
                user_exists.first_name = owner.user.first_name
                user_exists.last_name = owner.user.last_name
                session.add(user_exists)
                print(f"User {owner.user.username} already exists in the database and was updated.")
            
            channel = await session.execute(
                select(Channel).filter_by(channel_id=chat_id)
            )
            channel_exists = channel.scalar()
            if not channel_exists:
                new_channel = Channel(
                    channel_id=chat_info.id,
                    linked_channel_id=chat_info.linked_chat_id,
                    owner_telegram_id=owner.user.id,
                    title=chat_info.title,
                    description=chat_info.description,
                    bot_added_at=datetime.now(timezone.utc)
                )
                session.add(new_channel)
            else:
                channel_exists.linked_channel_id=chat_info.linked_chat_id
                channel_exists.owner_telegram_id=owner.user.id
                channel_exists.title=chat_info.title,
                channel_exists.description=chat_info.description
                session.add(channel_exists)
            await session.commit()


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

@router.my_chat_member(ChatMemberUpdatedFilter(member_status_changed=IS_NOT_MEMBER >> ADMINISTRATOR))
async def bot_added_as_admin(event: ChatMemberUpdated):
    await update_channel_and_owner(event.chat.id)
        
            

@router.channel_post()
async def channel_linked_chat_changed(message: Message):
    await update_channel_and_owner(message.chat.id)

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
        chat = await bot.get_chat(chat_id)
        print(chat.linked_chat_id)
    
        if not user_to_ban:
            await message.answer("Пользователь не найден.")
            return

        await remove_user(chat_id, user_to_ban.user.id)
        await message.answer(f"Пользователь {user_to_ban.user.full_name} был забанен.")
    except Exception as e:
        await message.answer(f"Не удалось забанить пользователя. Ошибка: {e}")



async def main() -> None:
    dp.include_router(router)
    await db_initializer.init_database()
    await client.start()
    await dp.start_polling(bot)


if __name__ == '__main__':
    import asyncio
    asyncio.run(main())