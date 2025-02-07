from datetime import datetime
import uuid
from typing import Optional, List
from sqlalchemy import BigInteger, Text, Numeric, Boolean, DateTime, JSON, UUID,ForeignKey, create_engine, inspect, text
from sqlalchemy.orm import DeclarativeBase, relationship, Mapped, mapped_column
from sqlalchemy.sql import func
import os

# declarative base class
class Base(DeclarativeBase):
   pass

class User(Base):
   __tablename__ = 'users'

   telegram_id: Mapped[int] = mapped_column(BigInteger, primary_key=True)
   username: Mapped[str] = mapped_column(Text, nullable=False)
   first_name: Mapped[Optional[str]] = mapped_column(Text)
   last_name: Mapped[Optional[str]] = mapped_column(Text)
   created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False, server_default=func.now())
   last_active_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False, server_default=func.now())

   # Relationships
   channels: Mapped[List["Channel"]] = relationship("Channel", back_populates="owner")
   memberships: Mapped[List["ChannelMembership"]] = relationship("ChannelMembership", back_populates="user")
   payments: Mapped[List["PaymentTransaction"]] = relationship("PaymentTransaction", back_populates="user")
   invite_links: Mapped[List["InviteLink"]] = relationship("InviteLink", back_populates="user")

class Channel(Base):
   __tablename__ = 'channels'

   channel_id: Mapped[int] = mapped_column(BigInteger, primary_key=True)
   owner_telegram_id: Mapped[int] = mapped_column(BigInteger, ForeignKey('users.telegram_id'), nullable=False)
   title: Mapped[str] = mapped_column(Text, nullable=False)
   description: Mapped[str] = mapped_column(Text, nullable=False)
   monthly_price: Mapped[float] = mapped_column(Numeric, nullable=False)
   created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False, server_default=func.now())
   bot_added_at: Mapped[Optional[datetime]] = mapped_column(DateTime(timezone=True))
   settings: Mapped[dict] = mapped_column(JSON, nullable=False, server_default='{}')
   is_active: Mapped[bool] = mapped_column(Boolean, nullable=False, server_default='true')
   last_check_date: Mapped[Optional[datetime]] = mapped_column(DateTime(timezone=True))

   # Relationships
   owner: Mapped["User"] = relationship("User", back_populates="channels")
   memberships: Mapped[List["ChannelMembership"]] = relationship("ChannelMembership", back_populates="channel")
   payments: Mapped[List["PaymentTransaction"]] = relationship("PaymentTransaction", back_populates="channel")
   invite_links: Mapped[List["InviteLink"]] = relationship("InviteLink", back_populates="channel")

class ChannelMembership(Base):
   __tablename__ = 'channel_memberships'

   telegram_id: Mapped[int] = mapped_column(BigInteger, ForeignKey('users.telegram_id'), primary_key=True)
   channel_id: Mapped[int] = mapped_column(BigInteger, ForeignKey('channels.channel_id'), primary_key=True)
   subscription_start: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)
   subscription_end: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)
   payment_history: Mapped[list] = mapped_column(JSON, nullable=False, server_default='[]')
   notifications_sent: Mapped[list] = mapped_column(JSON, nullable=False, server_default='[]')
   status: Mapped[str] = mapped_column(Text, nullable=False)

   # Relationships
   user: Mapped["User"] = relationship("User", back_populates="memberships")
   channel: Mapped["Channel"] = relationship("Channel", back_populates="memberships")

class PaymentTransaction(Base):
   __tablename__ = 'payment_transactions'

   id: Mapped[uuid.UUID] = mapped_column(UUID, primary_key=True, default=uuid.uuid4)
   telegram_id: Mapped[int] = mapped_column(BigInteger, ForeignKey('users.telegram_id'), nullable=False)
   channel_id: Mapped[int] = mapped_column(BigInteger, ForeignKey('channels.channel_id'), nullable=False)
   amount: Mapped[float] = mapped_column(Numeric, nullable=False)
   currency: Mapped[str] = mapped_column(Text, nullable=False)
   status: Mapped[str] = mapped_column(Text, nullable=False)
   created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False, server_default=func.now())
   completed_at: Mapped[Optional[datetime]] = mapped_column(DateTime(timezone=True))
   transaction_data: Mapped[Optional[dict]] = mapped_column(JSON)

   # Relationships
   user: Mapped["User"] = relationship("User", back_populates="payments")
   channel: Mapped["Channel"] = relationship("Channel", back_populates="payments")

class InviteLink(Base):
   __tablename__ = 'invite_links'

   id: Mapped[uuid.UUID] = mapped_column(UUID, primary_key=True, default=uuid.uuid4)
   channel_id: Mapped[int] = mapped_column(BigInteger, ForeignKey('channels.channel_id'), nullable=False)
   user_id: Mapped[int] = mapped_column(BigInteger, ForeignKey('users.telegram_id'), nullable=False)
   expires_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)
   used: Mapped[bool] = mapped_column(Boolean, nullable=False, server_default='false')

   # Relationships
   channel: Mapped["Channel"] = relationship("Channel", back_populates="invite_links")
   user: Mapped["User"] = relationship("User", back_populates="invite_links")

   # Параметры подключения к PostgreSQL
DB_USER = os.getenv('POSTGRES_USER', 'postgres')
DB_PASSWORD = os.getenv('POSTGRES_PASSWORD', 'postgres')
DB_HOST = os.getenv('DB_HOST', 'localhost')
DB_PORT = os.getenv('DB_PORT', '5432')
DB_NAME = os.getenv('POSTGRES_DB', 'telegram_bot_db')

def check_database_exists():
    # Подключаемся к postgres для проверки существования базы данных
    engine = create_engine(f"postgresql://{DB_USER}:{DB_PASSWORD}@{DB_HOST}:{DB_PORT}/postgres")
    
    with engine.connect() as conn:
        # Проверяем существование базы данных
        result = conn.execute(text(
            f"SELECT 1 FROM pg_database WHERE datname = '{DB_NAME}'"
        ))
        exists = result.scalar() is not None
        
        if not exists:
            # Создаем базу данных если она не существует
            conn.execute(text(f"CREATE DATABASE {DB_NAME}"))
            conn.commit()
            print(f"Database {DB_NAME} created")
        else:
            print(f"Database {DB_NAME} already exists")
    
    engine.dispose()

def init_database():
    # Проверяем и создаем базу данных если нужно
    check_database_exists()
    
    # Подключаемся к созданной базе данных
    engine = create_engine(f"postgresql://{DB_USER}:{DB_PASSWORD}@{DB_HOST}:{DB_PORT}/{DB_NAME}")
    
    # Создаем все таблицы
    print("Creating tables...")
    Base.metadata.create_all(engine)
    
    # Проверяем созданные таблицы
    inspector = inspect(engine)
    print("\nCreated tables:")
    for table_name in inspector.get_table_names():
        print(f"- {table_name}")
    
    # Закрываем соединение
    engine.dispose()