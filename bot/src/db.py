from datetime import datetime
import uuid
from typing import AsyncGenerator, Optional, List
from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession, async_sessionmaker
from sqlalchemy import BigInteger, Text, Numeric, Boolean, DateTime, JSON, UUID,ForeignKey, create_engine, inspect, text
from sqlalchemy.orm import DeclarativeBase, relationship, Mapped, mapped_column
from sqlalchemy.sql import func
from sqlalchemy.exc import SQLAlchemyError
from dotenv import load_dotenv
import os
from contextlib import asynccontextmanager

load_dotenv()

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
   linked_channel_id: Mapped[int] = mapped_column(BigInteger, nullable=True)
   owner_telegram_id: Mapped[int] = mapped_column(BigInteger, ForeignKey('users.telegram_id'), nullable=False)
   title: Mapped[str] = mapped_column(Text, nullable=True)
   description: Mapped[str] = mapped_column(Text, nullable=True)
   monthly_price: Mapped[float] = mapped_column(Numeric, nullable=True)
   created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False, server_default=func.now())
   bot_added_at: Mapped[Optional[datetime]] = mapped_column(DateTime(timezone=True))
   settings: Mapped[dict] = mapped_column(JSON, nullable=False, server_default='{}')
   is_active: Mapped[bool] = mapped_column(Boolean, nullable=False, server_default='true')
   last_check_date: Mapped[Optional[datetime]] = mapped_column(DateTime(timezone=True), nullable=True, server_default=func.now())

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


class DatabaseConfig:
   """Database configuration and connection management."""
    
   def __init__(self):
      load_dotenv()
      self.user = os.getenv('POSTGRES_USER', 'postgres')
      self.password = os.getenv('POSTGRES_PASSWORD', 'postgres')
      self.host = os.getenv('DB_HOST', 'localhost')
      self.port = os.getenv('DB_PORT', '5432')
      self.name = os.getenv('POSTGRES_DB', 'telegram_bot_db')
         
      self.engine = create_async_engine(
            f"postgresql+asyncpg://{self.user}:{self.password}@{self.host}:{self.port}/{self.name}",
            isolation_level="AUTOCOMMIT",
            echo=False,
            pool_size=5,
            max_overflow=10
         )
         
      self.async_session = async_sessionmaker(self.engine, class_=AsyncSession, expire_on_commit=False)

   async def get_session(self) -> AsyncGenerator[AsyncSession, None]:
      """Get database session with automatic cleanup."""
      session = self.async_session()
      try:
         yield session
      finally:
         await session.close()

class DatabaseInitializer:
   """Handle database initialization and table creation."""
    
   def __init__(self, config: DatabaseConfig):
      self.config = config
         
   async def check_database_exists(self) -> None:
      """Check if database exists and create if it doesn't."""
      try:
         async with self.config.engine.begin() as conn:
            result = await conn.execute(text(f"SELECT 1 FROM pg_database WHERE datname = '{self.config.name}'"))
            exists = result.scalar() is not None
                  
            if not exists:
               await conn.execute(text(f"CREATE DATABASE {self.config.name}"))
               print(f"Database {self.config.name} created successfully")
            else:
               print(f"Database {self.config.name} already exists")
      except SQLAlchemyError as e:
         print(f"Error checking/creating database: {e}")
         raise

   async def init_tables(self) -> None:
      """Initialize database tables."""
      try:
         async with self.config.engine.begin() as conn:
            print("Creating database tables...")
            await conn.run_sync(Base.metadata.create_all)
                  
         async with self.config.engine.begin() as conn:
            tables = await conn.run_sync(lambda sync_conn: inspect(sync_conn).get_table_names())
            print("Created tables: %s", ", ".join(tables))
      except SQLAlchemyError as e:
         print(f"Error creating tables: {e}")
         raise

   async def init_database(self) -> None:
      """Initialize complete database setup."""
      try:
         await self.check_database_exists()
         await self.init_tables()
         print("Database initialization completed successfully")
      except Exception as e:
         print(f"Database initialization failed: {e}")
         raise
      
db_config = DatabaseConfig()
db_initializer = DatabaseInitializer(db_config)