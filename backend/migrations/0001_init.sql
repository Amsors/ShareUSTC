-- ShareUSTC 初始数据库 schema（迁移基线，阶段3.1）
-- 由线上库 `pg_dump --schema-only --no-owner --no-privileges` 导出并规范化而来，
-- 与生产 schema 精确一致；后续 schema 变更一律新增迁移文件，禁止修改本文件。
-- 存量库首次接入 sqlx migrate 时需执行 `sqlx migrate resolve --version 1` 标记基线，
-- 避免在已有表的库上重复建表（详见 docs/deploy_guide.md）。

--
--




--
-- Name: public; Type: SCHEMA; Schema: -; Owner: -
--

-- *not* creating schema, since initdb creates it


--
-- Name: pgcrypto; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;


--
-- Name: update_updated_at_column(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_updated_at_column() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$;




--
-- Name: audit_logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.audit_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid,
    action character varying(100) NOT NULL,
    target_type character varying(50),
    target_id uuid,
    details jsonb DEFAULT '{}'::jsonb,
    ip_address inet,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: claims; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.claims (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    resource_id uuid NOT NULL,
    applicant_id uuid NOT NULL,
    claim_type character varying(20),
    reason text NOT NULL,
    proof_files jsonb DEFAULT '[]'::jsonb,
    status character varying(20) DEFAULT 'pending'::character varying,
    reviewer_id uuid,
    reviewed_at timestamp without time zone,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: comments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.comments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    resource_id uuid NOT NULL,
    user_id uuid NOT NULL,
    content text NOT NULL,
    audit_status character varying(20) DEFAULT 'approved'::character varying,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: course_sn_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.course_sn_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: courses; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.courses (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    sn bigint DEFAULT nextval('public.course_sn_seq'::regclass) NOT NULL,
    name character varying(255) DEFAULT ''::character varying NOT NULL,
    semester character varying(50),
    credits double precision,
    is_active boolean DEFAULT true,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: download_logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.download_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    resource_id uuid NOT NULL,
    user_id uuid,
    ip_address inet NOT NULL,
    downloaded_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: favorite_resources; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.favorite_resources (
    favorite_id uuid NOT NULL,
    resource_id uuid NOT NULL,
    added_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: favorites; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.favorites (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: images; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.images (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    uploader_id uuid NOT NULL,
    file_path character varying(500) NOT NULL,
    original_name character varying(255),
    file_size integer,
    mime_type character varying(50),
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    storage_type character varying(20) DEFAULT 'local'::character varying,
    file_url character varying(1000)
);


--
-- Name: likes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.likes (
    resource_id uuid NOT NULL,
    user_id uuid NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: notification_reads; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.notification_reads (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    notification_id uuid NOT NULL,
    user_id uuid NOT NULL,
    read_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: notifications; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.notifications (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    recipient_id uuid,
    title character varying(255) NOT NULL,
    content text NOT NULL,
    notification_type character varying(50),
    priority character varying(20) DEFAULT 'normal'::character varying,
    is_read boolean DEFAULT false,
    link_url character varying(500),
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: ratings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ratings (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    resource_id uuid NOT NULL,
    user_id uuid NOT NULL,
    difficulty integer,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    overall_quality integer,
    answer_quality integer,
    format_quality integer,
    detail_level integer,
    CONSTRAINT ratings_answer_quality_check CHECK (((answer_quality >= 1) AND (answer_quality <= 10))),
    CONSTRAINT ratings_detail_level_check CHECK (((detail_level >= 1) AND (detail_level <= 10))),
    CONSTRAINT ratings_difficulty_check CHECK (((difficulty >= 1) AND (difficulty <= 10))),
    CONSTRAINT ratings_format_quality_check CHECK (((format_quality >= 1) AND (format_quality <= 10))),
    CONSTRAINT ratings_overall_quality_check CHECK (((overall_quality >= 1) AND (overall_quality <= 10)))
);


--
-- Name: resource_courses; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_courses (
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    resource_id uuid NOT NULL,
    course_sn bigint NOT NULL
);


--
-- Name: resource_relations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_relations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    source_resource_id uuid DEFAULT '00000000-0000-0000-0000-000000000000'::uuid NOT NULL,
    target_resource_id uuid DEFAULT '00000000-0000-0000-0000-000000000000'::uuid NOT NULL
);


--
-- Name: resource_stats; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_stats (
    resource_id uuid NOT NULL,
    views integer DEFAULT 0,
    downloads integer DEFAULT 0,
    likes integer DEFAULT 0,
    difficulty_total integer DEFAULT 0,
    difficulty_count integer DEFAULT 0,
    overall_quality_total integer DEFAULT 0,
    overall_quality_count integer DEFAULT 0,
    answer_quality_total integer DEFAULT 0,
    answer_quality_count integer DEFAULT 0,
    format_quality_total integer DEFAULT 0,
    format_quality_count integer DEFAULT 0,
    detail_level_total integer DEFAULT 0,
    detail_level_count integer DEFAULT 0,
    rating_count integer DEFAULT 0
);


--
-- Name: resource_teachers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resource_teachers (
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    resource_id uuid NOT NULL,
    teacher_sn bigint NOT NULL
);


--
-- Name: resources; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.resources (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    title character varying(255) NOT NULL,
    author_id uuid,
    uploader_id uuid NOT NULL,
    course_name character varying(255),
    resource_type character varying(50),
    category character varying(50),
    tags jsonb DEFAULT '[]'::jsonb,
    file_path character varying(500),
    source_file_path character varying(500),
    file_hash character varying(64),
    file_size bigint,
    content_accuracy double precision,
    audit_status character varying(20) DEFAULT 'pending'::character varying,
    ai_reject_reason text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    storage_type character varying(20) DEFAULT 'local'::character varying,
    file_url character varying(1000),
    source_file_url character varying(1000),
    description text
);


--
-- Name: teacher_sn_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.teacher_sn_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: teachers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.teachers (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    sn bigint DEFAULT nextval('public.teacher_sn_seq'::regclass) NOT NULL,
    name character varying(100) DEFAULT ''::character varying NOT NULL,
    department character varying(100),
    is_active boolean DEFAULT true,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: user_sn_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_sn_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    sn bigint,
    username character varying(50) NOT NULL,
    password_hash character varying(255) NOT NULL,
    email character varying(255),
    role character varying(20) DEFAULT 'user'::character varying,
    bio text,
    social_links jsonb DEFAULT '{}'::jsonb,
    real_info jsonb DEFAULT '{}'::jsonb,
    is_verified boolean DEFAULT false,
    is_active boolean DEFAULT true,
    avatar_url character varying(500),
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: audit_logs audit_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs
    ADD CONSTRAINT audit_logs_pkey PRIMARY KEY (id);


--
-- Name: claims claims_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.claims
    ADD CONSTRAINT claims_pkey PRIMARY KEY (id);


--
-- Name: comments comments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_pkey PRIMARY KEY (id);


--
-- Name: courses courses_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.courses
    ADD CONSTRAINT courses_pkey PRIMARY KEY (id);


--
-- Name: courses courses_sn_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.courses
    ADD CONSTRAINT courses_sn_key UNIQUE (sn);


--
-- Name: download_logs download_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.download_logs
    ADD CONSTRAINT download_logs_pkey PRIMARY KEY (id);


--
-- Name: favorite_resources favorite_resources_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.favorite_resources
    ADD CONSTRAINT favorite_resources_pkey PRIMARY KEY (favorite_id, resource_id);


--
-- Name: favorites favorites_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.favorites
    ADD CONSTRAINT favorites_pkey PRIMARY KEY (id);


--
-- Name: images images_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.images
    ADD CONSTRAINT images_pkey PRIMARY KEY (id);


--
-- Name: likes likes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.likes
    ADD CONSTRAINT likes_pkey PRIMARY KEY (resource_id, user_id);


--
-- Name: notification_reads notification_reads_notification_id_user_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notification_reads
    ADD CONSTRAINT notification_reads_notification_id_user_id_key UNIQUE (notification_id, user_id);


--
-- Name: notification_reads notification_reads_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notification_reads
    ADD CONSTRAINT notification_reads_pkey PRIMARY KEY (id);


--
-- Name: notifications notifications_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_pkey PRIMARY KEY (id);


--
-- Name: ratings ratings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ratings
    ADD CONSTRAINT ratings_pkey PRIMARY KEY (id);


--
-- Name: ratings ratings_resource_id_user_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ratings
    ADD CONSTRAINT ratings_resource_id_user_id_key UNIQUE (resource_id, user_id);


--
-- Name: resource_courses resource_courses_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_courses
    ADD CONSTRAINT resource_courses_pkey PRIMARY KEY (resource_id, course_sn);


--
-- Name: resource_relations resource_relations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_relations
    ADD CONSTRAINT resource_relations_pkey PRIMARY KEY (id);


--
-- Name: resource_relations resource_relations_source_target_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_relations
    ADD CONSTRAINT resource_relations_source_target_key UNIQUE (source_resource_id, target_resource_id);


--
-- Name: resource_stats resource_stats_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_stats
    ADD CONSTRAINT resource_stats_pkey PRIMARY KEY (resource_id);


--
-- Name: resource_teachers resource_teachers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_teachers
    ADD CONSTRAINT resource_teachers_pkey PRIMARY KEY (resource_id, teacher_sn);


--
-- Name: resources resources_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resources
    ADD CONSTRAINT resources_pkey PRIMARY KEY (id);


--
-- Name: teachers teachers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.teachers
    ADD CONSTRAINT teachers_pkey PRIMARY KEY (id);


--
-- Name: teachers teachers_sn_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.teachers
    ADD CONSTRAINT teachers_sn_key UNIQUE (sn);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users users_sn_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_sn_key UNIQUE (sn);


--
-- Name: users users_username_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_username_key UNIQUE (username);


--
-- Name: idx_audit_logs_action; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_action ON public.audit_logs USING btree (action);


--
-- Name: idx_audit_logs_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_created_at ON public.audit_logs USING btree (created_at DESC);


--
-- Name: idx_audit_logs_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_user ON public.audit_logs USING btree (user_id);


--
-- Name: idx_claims_applicant; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_claims_applicant ON public.claims USING btree (applicant_id);


--
-- Name: idx_claims_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_claims_resource ON public.claims USING btree (resource_id);


--
-- Name: idx_claims_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_claims_status ON public.claims USING btree (status);


--
-- Name: idx_comments_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comments_created_at ON public.comments USING btree (created_at DESC);


--
-- Name: idx_comments_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comments_resource ON public.comments USING btree (resource_id);


--
-- Name: idx_comments_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comments_user ON public.comments USING btree (user_id);


--
-- Name: idx_courses_is_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_courses_is_active ON public.courses USING btree (is_active);


--
-- Name: idx_courses_semester; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_courses_semester ON public.courses USING btree (semester);


--
-- Name: idx_courses_sn; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_courses_sn ON public.courses USING btree (sn);


--
-- Name: idx_download_logs_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_download_logs_resource ON public.download_logs USING btree (resource_id);


--
-- Name: idx_download_logs_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_download_logs_time ON public.download_logs USING btree (downloaded_at DESC);


--
-- Name: idx_download_logs_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_download_logs_user ON public.download_logs USING btree (user_id);


--
-- Name: idx_fav_res_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fav_res_resource ON public.favorite_resources USING btree (resource_id);


--
-- Name: idx_favorites_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_favorites_user ON public.favorites USING btree (user_id);


--
-- Name: idx_images_storage_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_images_storage_type ON public.images USING btree (storage_type);


--
-- Name: idx_images_uploader; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_images_uploader ON public.images USING btree (uploader_id);


--
-- Name: idx_likes_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_likes_user ON public.likes USING btree (user_id);


--
-- Name: idx_notification_reads_notification; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notification_reads_notification ON public.notification_reads USING btree (notification_id);


--
-- Name: idx_notification_reads_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notification_reads_unique ON public.notification_reads USING btree (notification_id, user_id);


--
-- Name: idx_notification_reads_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notification_reads_user ON public.notification_reads USING btree (user_id);


--
-- Name: idx_notifications_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_created_at ON public.notifications USING btree (created_at DESC);


--
-- Name: idx_notifications_is_read; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_is_read ON public.notifications USING btree (is_read);


--
-- Name: idx_notifications_priority; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_priority ON public.notifications USING btree (priority);


--
-- Name: idx_notifications_recipient; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_recipient ON public.notifications USING btree (recipient_id);


--
-- Name: idx_ratings_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ratings_resource ON public.ratings USING btree (resource_id);


--
-- Name: idx_ratings_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ratings_user ON public.ratings USING btree (user_id);


--
-- Name: idx_resource_courses_course; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_courses_course ON public.resource_courses USING btree (course_sn);


--
-- Name: idx_resource_courses_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_courses_resource ON public.resource_courses USING btree (resource_id);


--
-- Name: idx_resource_relations_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_relations_source ON public.resource_relations USING btree (source_resource_id);


--
-- Name: idx_resource_relations_target; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_relations_target ON public.resource_relations USING btree (target_resource_id);


--
-- Name: idx_resource_stats_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_stats_resource ON public.resource_stats USING btree (resource_id);


--
-- Name: idx_resource_teachers_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_teachers_resource ON public.resource_teachers USING btree (resource_id);


--
-- Name: idx_resource_teachers_teacher; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_teachers_teacher ON public.resource_teachers USING btree (teacher_sn);


--
-- Name: idx_resources_audit_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_audit_status ON public.resources USING btree (audit_status);


--
-- Name: idx_resources_author; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_author ON public.resources USING btree (author_id);


--
-- Name: idx_resources_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_category ON public.resources USING btree (category);


--
-- Name: idx_resources_course; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_course ON public.resources USING btree (course_name);


--
-- Name: idx_resources_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_created_at ON public.resources USING btree (created_at DESC);


--
-- Name: idx_resources_storage_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_storage_type ON public.resources USING btree (storage_type);


--
-- Name: idx_resources_tags; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_tags ON public.resources USING gin (tags);


--
-- Name: idx_resources_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_type ON public.resources USING btree (resource_type);


--
-- Name: idx_resources_uploader; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_uploader ON public.resources USING btree (uploader_id);


--
-- Name: idx_teachers_department; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_teachers_department ON public.teachers USING btree (department);


--
-- Name: idx_teachers_is_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_teachers_is_active ON public.teachers USING btree (is_active);


--
-- Name: idx_teachers_sn; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_teachers_sn ON public.teachers USING btree (sn);


--
-- Name: idx_users_is_verified; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_is_verified ON public.users USING btree (is_verified);


--
-- Name: idx_users_role; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_role ON public.users USING btree (role);


--
-- Name: idx_users_sn; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_sn ON public.users USING btree (sn);


--
-- Name: comments update_comments_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_comments_updated_at BEFORE UPDATE ON public.comments FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: courses update_courses_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_courses_updated_at BEFORE UPDATE ON public.courses FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: ratings update_ratings_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_ratings_updated_at BEFORE UPDATE ON public.ratings FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: resources update_resources_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_resources_updated_at BEFORE UPDATE ON public.resources FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: teachers update_teachers_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_teachers_updated_at BEFORE UPDATE ON public.teachers FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: users update_users_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON public.users FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: audit_logs audit_logs_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs
    ADD CONSTRAINT audit_logs_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: claims claims_applicant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.claims
    ADD CONSTRAINT claims_applicant_id_fkey FOREIGN KEY (applicant_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: claims claims_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.claims
    ADD CONSTRAINT claims_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resources(id) ON DELETE CASCADE;


--
-- Name: claims claims_reviewer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.claims
    ADD CONSTRAINT claims_reviewer_id_fkey FOREIGN KEY (reviewer_id) REFERENCES public.users(id);


--
-- Name: comments comments_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resources(id) ON DELETE CASCADE;


--
-- Name: comments comments_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: download_logs download_logs_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.download_logs
    ADD CONSTRAINT download_logs_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resources(id) ON DELETE CASCADE;


--
-- Name: download_logs download_logs_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.download_logs
    ADD CONSTRAINT download_logs_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: favorite_resources favorite_resources_favorite_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.favorite_resources
    ADD CONSTRAINT favorite_resources_favorite_id_fkey FOREIGN KEY (favorite_id) REFERENCES public.favorites(id) ON DELETE CASCADE;


--
-- Name: favorite_resources favorite_resources_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.favorite_resources
    ADD CONSTRAINT favorite_resources_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resources(id) ON DELETE CASCADE;


--
-- Name: favorites favorites_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.favorites
    ADD CONSTRAINT favorites_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: images images_uploader_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.images
    ADD CONSTRAINT images_uploader_id_fkey FOREIGN KEY (uploader_id) REFERENCES public.users(id);


--
-- Name: likes likes_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.likes
    ADD CONSTRAINT likes_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resources(id) ON DELETE CASCADE;


--
-- Name: likes likes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.likes
    ADD CONSTRAINT likes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: notification_reads notification_reads_notification_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notification_reads
    ADD CONSTRAINT notification_reads_notification_id_fkey FOREIGN KEY (notification_id) REFERENCES public.notifications(id) ON DELETE CASCADE;


--
-- Name: notification_reads notification_reads_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notification_reads
    ADD CONSTRAINT notification_reads_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: notifications notifications_recipient_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_recipient_id_fkey FOREIGN KEY (recipient_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: ratings ratings_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ratings
    ADD CONSTRAINT ratings_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resources(id) ON DELETE CASCADE;


--
-- Name: ratings ratings_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ratings
    ADD CONSTRAINT ratings_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: resource_courses resource_courses_course_sn_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_courses
    ADD CONSTRAINT resource_courses_course_sn_fkey FOREIGN KEY (course_sn) REFERENCES public.courses(sn) ON DELETE CASCADE;


--
-- Name: resource_courses resource_courses_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_courses
    ADD CONSTRAINT resource_courses_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resources(id) ON DELETE CASCADE;


--
-- Name: resource_relations resource_relations_source_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_relations
    ADD CONSTRAINT resource_relations_source_resource_id_fkey FOREIGN KEY (source_resource_id) REFERENCES public.resources(id) ON DELETE CASCADE;


--
-- Name: resource_relations resource_relations_target_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_relations
    ADD CONSTRAINT resource_relations_target_resource_id_fkey FOREIGN KEY (target_resource_id) REFERENCES public.resources(id) ON DELETE CASCADE;


--
-- Name: resource_stats resource_stats_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_stats
    ADD CONSTRAINT resource_stats_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resources(id) ON DELETE CASCADE;


--
-- Name: resource_teachers resource_teachers_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_teachers
    ADD CONSTRAINT resource_teachers_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES public.resources(id) ON DELETE CASCADE;


--
-- Name: resource_teachers resource_teachers_teacher_sn_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resource_teachers
    ADD CONSTRAINT resource_teachers_teacher_sn_fkey FOREIGN KEY (teacher_sn) REFERENCES public.teachers(sn) ON DELETE CASCADE;


--
-- Name: resources resources_author_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resources
    ADD CONSTRAINT resources_author_id_fkey FOREIGN KEY (author_id) REFERENCES public.users(id);


--
-- Name: resources resources_uploader_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.resources
    ADD CONSTRAINT resources_uploader_id_fkey FOREIGN KEY (uploader_id) REFERENCES public.users(id);


--
--


