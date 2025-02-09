import os
from aiogram import Bot, Dispatcher, Router, F
from aiogram.types import Message, ChatMemberUpdated, ChatMemberUpdated
from aiogram.filters import Command, BaseFilter, ChatMemberUpdatedFilter, IS_NOT_MEMBER, MEMBER, ADMINISTRATOR
from dotenv import load_dotenv
from db import init_database
from telethon import TelegramClient
from telethon.tl.types import PeerUser
from telethon.tl.functions.channels import GetParticipantRequest
from sqlalchemy.orm import sessionmaker
from db import User, async_engine

load_dotenv()


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
    admins = await chat_info.get_administrators()

    # Find the owner of the chat
    owner = next((admin for admin in admins if admin.status == 'creator'), None)
    # if owner:
        # Database connection
        # Session = sessionmaker(bind=async_engine)
        # session = Session()

        # Check if the user already exists in the database
        # user_exists = session.query(User).filter_by(telegram_id=owner.user.id).first()
        # if not user_exists:
            # Add the owner to the database
        #    new_user = User(
        #        telegram_id=owner.user.id,
        #        username=owner.user.username,
        #        first_name=owner.user.first_name,
        #        last_name=owner.user.last_name
        #    )
        #    session.add(new_user)
        #    session.commit()
        #    print(f"User {owner.user.username} added to the database.")
        # else:
        #    print(f"User {owner.user.username} already exists in the database.")

        # session.close()

@router.channel_post()
async def channel_linked_chat_changed(message: Message):
    print("Как-нибудь логируем эту ситуацию")


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
    await init_database()
    await client.start()
    await dp.start_polling(bot)


if __name__ == '__main__':
    import asyncio
    asyncio.run(main())