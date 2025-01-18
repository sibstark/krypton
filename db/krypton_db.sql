--
-- PostgreSQL database dump
--

-- Dumped from database version 17.2 (Debian 17.2-1.pgdg120+1)
-- Dumped by pg_dump version 17.2

-- Started on 2025-01-18 11:35:24 UTC

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- TOC entry 4 (class 2615 OID 2200)
-- Name: public; Type: SCHEMA; Schema: -; Owner: pg_database_owner
--

CREATE SCHEMA public;


ALTER SCHEMA public OWNER TO pg_database_owner;

--
-- TOC entry 3403 (class 0 OID 0)
-- Dependencies: 4
-- Name: SCHEMA public; Type: COMMENT; Schema: -; Owner: pg_database_owner
--

COMMENT ON SCHEMA public IS 'standard public schema';


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- TOC entry 219 (class 1259 OID 16413)
-- Name: channel_memberships; Type: TABLE; Schema: public; Owner: admin
--

CREATE TABLE public.channel_memberships (
    telegram_id bigint NOT NULL,
    channel_id bigint NOT NULL,
    subscription_start timestamp with time zone NOT NULL,
    subscription_end timestamp with time zone NOT NULL,
    payment_history jsonb DEFAULT '[]'::jsonb NOT NULL,
    notifications_sent jsonb DEFAULT '[]'::jsonb NOT NULL,
    status text NOT NULL
);


ALTER TABLE public.channel_memberships OWNER TO admin;

--
-- TOC entry 218 (class 1259 OID 16398)
-- Name: channels; Type: TABLE; Schema: public; Owner: admin
--

CREATE TABLE public.channels (
    channel_id bigint NOT NULL,
    owner_telegram_id bigint NOT NULL,
    title text NOT NULL,
    description text NOT NULL,
    monthly_price numeric NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    bot_added_at timestamp with time zone,
    settings jsonb DEFAULT '{}'::jsonb NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    last_check_date timestamp with time zone
);


ALTER TABLE public.channels OWNER TO admin;

--
-- TOC entry 221 (class 1259 OID 16451)
-- Name: invite_links; Type: TABLE; Schema: public; Owner: admin
--

CREATE TABLE public.invite_links (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    channel_id bigint NOT NULL,
    user_id bigint NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    used boolean DEFAULT false NOT NULL
);


ALTER TABLE public.invite_links OWNER TO admin;

--
-- TOC entry 220 (class 1259 OID 16432)
-- Name: payment_transactions; Type: TABLE; Schema: public; Owner: admin
--

CREATE TABLE public.payment_transactions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    telegram_id bigint NOT NULL,
    channel_id bigint NOT NULL,
    amount numeric NOT NULL,
    currency text NOT NULL,
    status text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    completed_at timestamp with time zone,
    transaction_data jsonb
);


ALTER TABLE public.payment_transactions OWNER TO admin;

--
-- TOC entry 217 (class 1259 OID 16389)
-- Name: users; Type: TABLE; Schema: public; Owner: admin
--

CREATE TABLE public.users (
    telegram_id bigint NOT NULL,
    username text NOT NULL,
    first_name text,
    last_name text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    last_active_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.users OWNER TO admin;

--
-- TOC entry 3241 (class 2606 OID 16421)
-- Name: channel_memberships channel_memberships_pkey; Type: CONSTRAINT; Schema: public; Owner: admin
--

ALTER TABLE ONLY public.channel_memberships
    ADD CONSTRAINT channel_memberships_pkey PRIMARY KEY (telegram_id, channel_id);


--
-- TOC entry 3239 (class 2606 OID 16407)
-- Name: channels channels_pkey; Type: CONSTRAINT; Schema: public; Owner: admin
--

ALTER TABLE ONLY public.channels
    ADD CONSTRAINT channels_pkey PRIMARY KEY (channel_id);


--
-- TOC entry 3245 (class 2606 OID 16457)
-- Name: invite_links invite_links_pkey; Type: CONSTRAINT; Schema: public; Owner: admin
--

ALTER TABLE ONLY public.invite_links
    ADD CONSTRAINT invite_links_pkey PRIMARY KEY (id);


--
-- TOC entry 3243 (class 2606 OID 16440)
-- Name: payment_transactions payment_transactions_pkey; Type: CONSTRAINT; Schema: public; Owner: admin
--

ALTER TABLE ONLY public.payment_transactions
    ADD CONSTRAINT payment_transactions_pkey PRIMARY KEY (id);


--
-- TOC entry 3237 (class 2606 OID 16397)
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: admin
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (telegram_id);


--
-- TOC entry 3247 (class 2606 OID 16427)
-- Name: channel_memberships channel_memberships_channel_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: admin
--

ALTER TABLE ONLY public.channel_memberships
    ADD CONSTRAINT channel_memberships_channel_id_fkey FOREIGN KEY (channel_id) REFERENCES public.channels(channel_id);


--
-- TOC entry 3248 (class 2606 OID 16422)
-- Name: channel_memberships channel_memberships_telegram_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: admin
--

ALTER TABLE ONLY public.channel_memberships
    ADD CONSTRAINT channel_memberships_telegram_id_fkey FOREIGN KEY (telegram_id) REFERENCES public.users(telegram_id);


--
-- TOC entry 3246 (class 2606 OID 16408)
-- Name: channels channels_owner_telegram_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: admin
--

ALTER TABLE ONLY public.channels
    ADD CONSTRAINT channels_owner_telegram_id_fkey FOREIGN KEY (owner_telegram_id) REFERENCES public.users(telegram_id);


--
-- TOC entry 3251 (class 2606 OID 16458)
-- Name: invite_links invite_links_channel_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: admin
--

ALTER TABLE ONLY public.invite_links
    ADD CONSTRAINT invite_links_channel_id_fkey FOREIGN KEY (channel_id) REFERENCES public.channels(channel_id);


--
-- TOC entry 3252 (class 2606 OID 16463)
-- Name: invite_links invite_links_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: admin
--

ALTER TABLE ONLY public.invite_links
    ADD CONSTRAINT invite_links_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(telegram_id);


--
-- TOC entry 3249 (class 2606 OID 16446)
-- Name: payment_transactions payment_transactions_channel_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: admin
--

ALTER TABLE ONLY public.payment_transactions
    ADD CONSTRAINT payment_transactions_channel_id_fkey FOREIGN KEY (channel_id) REFERENCES public.channels(channel_id);


--
-- TOC entry 3250 (class 2606 OID 16441)
-- Name: payment_transactions payment_transactions_telegram_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: admin
--

ALTER TABLE ONLY public.payment_transactions
    ADD CONSTRAINT payment_transactions_telegram_id_fkey FOREIGN KEY (telegram_id) REFERENCES public.users(telegram_id);


-- Completed on 2025-01-18 11:35:24 UTC

--
-- PostgreSQL database dump complete
--

