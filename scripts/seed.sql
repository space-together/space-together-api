-- Default demo seed: one row in every table.
-- Idempotent: re-runnable thanks to ON CONFLICT DO NOTHING.
-- IDs are 24 chars (DB CHECK) via rpad('<label>', 24, '0').

BEGIN;

-- ---------- identity ----------
INSERT INTO users (id, name, email, username, role) VALUES
  (rpad('u-admin',24,'0'),   'Demo Admin',   'admin@demo.school',   'demo_admin',   'admin'),
  (rpad('u-teacher',24,'0'), 'Demo Teacher', 'teacher@demo.school', 'demo_teacher', 'teacher'),
  (rpad('u-student',24,'0'), 'Demo Student', 'student@demo.school', 'demo_student', 'student'),
  (rpad('u-parent',24,'0'),  'Demo Parent',  'parent@demo.school',  'demo_parent',  'parent'),
  (rpad('u-staff',24,'0'),   'Demo Staff',   'staff@demo.school',   'demo_staff',   'staff')
ON CONFLICT DO NOTHING;

INSERT INTO schools (id, creator_id, username, name, code, school_type, description) VALUES
  (rpad('s-main',24,'0'), rpad('u-admin',24,'0'), 'demo-school', 'Demo School', 'DEMO', 'secondary', 'Default seeded school')
ON CONFLICT DO NOTHING;

-- ---------- reference catalogs ----------
INSERT INTO permissions (id, code, description) VALUES
  (rpad('perm-1',24,'0'), 'school.manage', 'Manage school settings')
ON CONFLICT DO NOTHING;

INSERT INTO roles (id, school_id, name, code, description) VALUES
  (rpad('role-1',24,'0'), rpad('s-main',24,'0'), 'Administrator', 'admin', 'Default admin role')
ON CONFLICT DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id) VALUES
  (rpad('role-1',24,'0'), rpad('perm-1',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO sectors (id, name, username, country, type, description) VALUES
  (rpad('sec-1',24,'0'), 'Science', 'science', 'Rwanda', 'general', 'Science sector')
ON CONFLICT DO NOTHING;

INSERT INTO trades (id, sector_id, name, username, description) VALUES
  (rpad('trd-1',24,'0'), rpad('sec-1',24,'0'), 'Software Development', 'swdev', 'SWD trade')
ON CONFLICT DO NOTHING;

INSERT INTO main_classes (id, name, username, trade_id, level, description) VALUES
  (rpad('mcls-1',24,'0'), 'Senior 1', 's1', rpad('trd-1',24,'0'), 1, 'Main class S1')
ON CONFLICT DO NOTHING;

INSERT INTO template_subjects (id, name, code) VALUES
  (rpad('tmpl-1',24,'0'), 'Mathematics', 'MATH')
ON CONFLICT DO NOTHING;

-- ---------- people ----------
INSERT INTO student_profiles (id, user_id, name, email, gender) VALUES
  (rpad('sp-1',24,'0'), rpad('u-student',24,'0'), 'Demo Student', 'student@demo.school', 'male')
ON CONFLICT DO NOTHING;

INSERT INTO teachers (id, school_id, user_id, name, email, department, job_title) VALUES
  (rpad('tch-1',24,'0'), rpad('s-main',24,'0'), rpad('u-teacher',24,'0'), 'Demo Teacher', 'teacher@demo.school', 'Sciences', 'Teacher')
ON CONFLICT DO NOTHING;

INSERT INTO parents (id, school_id, user_id, student_id, name, email, relationship) VALUES
  (rpad('par-1',24,'0'), rpad('s-main',24,'0'), rpad('u-parent',24,'0'), rpad('sp-1',24,'0'), 'Demo Parent', 'parent@demo.school', 'father')
ON CONFLICT DO NOTHING;

INSERT INTO school_staff (id, school_id, user_id, name, email, department, job_title) VALUES
  (rpad('stf-1',24,'0'), rpad('s-main',24,'0'), rpad('u-staff',24,'0'), 'Demo Staff', 'staff@demo.school', 'Admin', 'Secretary')
ON CONFLICT DO NOTHING;

-- ---------- academic structure ----------
INSERT INTO classes (id, school_id, creator_id, name, username, code, level, main_class_id, trade_id, class_teacher_id) VALUES
  (rpad('cls-1',24,'0'), rpad('s-main',24,'0'), rpad('u-admin',24,'0'), 'S1 A', 's1a', 'S1A', 'S1', rpad('mcls-1',24,'0'), rpad('trd-1',24,'0'), rpad('tch-1',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO class_subjects (id, school_id, class_id, teacher_user_id, teacher_id, name, code, category) VALUES
  (rpad('csub-1',24,'0'), rpad('s-main',24,'0'), rpad('cls-1',24,'0'), rpad('u-teacher',24,'0'), rpad('tch-1',24,'0'), 'Mathematics', 'MATH', 'core')
ON CONFLICT DO NOTHING;

INSERT INTO education_years (id, school_id, name, starts_on, ends_on, created_by) VALUES
  (rpad('eyear-1',24,'0'), rpad('s-main',24,'0'), '2025-2026', '2025-09-01', '2026-07-15', rpad('u-admin',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO terms (id, school_id, education_year_id, name, starts_on, ends_on) VALUES
  (rpad('term-1',24,'0'), rpad('s-main',24,'0'), rpad('eyear-1',24,'0'), 'Term 1', '2025-09-01', '2025-12-15')
ON CONFLICT DO NOTHING;

INSERT INTO student_school_enrollments (id, student_id, school_id, class_id, registration_number, admission_year, creator_id) VALUES
  (rpad('enr-1',24,'0'), rpad('sp-1',24,'0'), rpad('s-main',24,'0'), rpad('cls-1',24,'0'), 'REG-001', 2025, rpad('u-admin',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO grading_scales (id, school_id, education_year_id, name, created_by) VALUES
  (rpad('gscale-1',24,'0'), rpad('s-main',24,'0'), rpad('eyear-1',24,'0'), 'Default Scale', rpad('u-admin',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO grading_scale_boundaries (id, grading_scale_id, grade, min_score, max_score, gpa_value, description) VALUES
  (rpad('gbound-1',24,'0'), rpad('gscale-1',24,'0'), 'A', 80, 100, 4.0, 'Excellent')
ON CONFLICT DO NOTHING;

INSERT INTO exams (id, school_id, class_id, education_year_id, term_id, name, term, academic_year, created_by) VALUES
  (rpad('exam-1',24,'0'), rpad('s-main',24,'0'), rpad('cls-1',24,'0'), rpad('eyear-1',24,'0'), rpad('term-1',24,'0'), 'Term 1 Exam', 'Term 1', '2025-2026', rpad('u-admin',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO assessment_categories (id, school_id, class_id, class_subject_id, education_year_id, name, weight, weight_percentage, created_by) VALUES
  (rpad('acat-1',24,'0'), rpad('s-main',24,'0'), rpad('cls-1',24,'0'), rpad('csub-1',24,'0'), rpad('eyear-1',24,'0'), 'Quizzes', 30, 30, rpad('u-admin',24,'0'))
ON CONFLICT DO NOTHING;

-- ---------- coursework ----------
INSERT INTO assignments (id, school_id, class_id, class_subject_id, subject_id, teacher_user_id, teacher_id, title, description) VALUES
  (rpad('asg-1',24,'0'), rpad('s-main',24,'0'), rpad('cls-1',24,'0'), rpad('csub-1',24,'0'), rpad('csub-1',24,'0'), rpad('u-teacher',24,'0'), rpad('tch-1',24,'0'), 'Algebra Homework', 'Solve exercises 1-10')
ON CONFLICT DO NOTHING;

INSERT INTO submissions (id, school_id, assignment_id, student_id, enrollment_id, submitted_by, graded_by, feedback) VALUES
  (rpad('sub-1',24,'0'), rpad('s-main',24,'0'), rpad('asg-1',24,'0'), rpad('sp-1',24,'0'), rpad('enr-1',24,'0'), rpad('u-student',24,'0'), rpad('tch-1',24,'0'), 'Good work')
ON CONFLICT DO NOTHING;

INSERT INTO scores (id, school_id, student_id, enrollment_id, class_id, class_subject_id, exam_id, assessment_category_id, education_year_id, score, max_score, term, academic_year, recorded_by, entered_by) VALUES
  (rpad('score-1',24,'0'), rpad('s-main',24,'0'), rpad('sp-1',24,'0'), rpad('enr-1',24,'0'), rpad('cls-1',24,'0'), rpad('csub-1',24,'0'), rpad('exam-1',24,'0'), rpad('acat-1',24,'0'), rpad('eyear-1',24,'0'), 85, 100, 'Term 1', '2025-2026', rpad('u-teacher',24,'0'), rpad('u-teacher',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO score_audit_logs (id, school_id, score_id, changed_by, old_score, new_score, reason) VALUES
  (rpad('saudit-1',24,'0'), rpad('s-main',24,'0'), rpad('score-1',24,'0'), rpad('u-teacher',24,'0'), 80, 85, 'Re-mark')
ON CONFLICT DO NOTHING;

INSERT INTO attendance (id, school_id, student_id, enrollment_id, class_id, date, status, recorded_by, note) VALUES
  (rpad('att-1',24,'0'), rpad('s-main',24,'0'), rpad('sp-1',24,'0'), rpad('enr-1',24,'0'), rpad('cls-1',24,'0'), '2025-09-15', 'present', rpad('u-teacher',24,'0'), 'On time')
ON CONFLICT DO NOTHING;

-- ---------- term results ----------
INSERT INTO student_term_results (id, school_id, student_id, enrollment_id, class_id, education_year_id, exam_id, term_id, term, academic_year, status) VALUES
  (rpad('str-1',24,'0'), rpad('s-main',24,'0'), rpad('sp-1',24,'0'), rpad('enr-1',24,'0'), rpad('cls-1',24,'0'), rpad('eyear-1',24,'0'), rpad('exam-1',24,'0'), rpad('term-1',24,'0'), 'Term 1', '2025-2026', 'final')
ON CONFLICT DO NOTHING;

INSERT INTO student_term_subject_results (id, result_id, class_subject_id, subject_name) VALUES
  (rpad('stsr-1',24,'0'), rpad('str-1',24,'0'), rpad('csub-1',24,'0'), 'Mathematics')
ON CONFLICT DO NOTHING;

INSERT INTO student_term_category_scores (id, subject_result_id, assessment_category_id, category_name) VALUES
  (rpad('stcs-1',24,'0'), rpad('stsr-1',24,'0'), rpad('acat-1',24,'0'), 'Quizzes')
ON CONFLICT DO NOTHING;

-- ---------- messaging ----------
INSERT INTO conversations (id, school_id, created_by, conversation_type, title, name) VALUES
  (rpad('conv-1',24,'0'), rpad('s-main',24,'0'), rpad('u-admin',24,'0'), 'group', 'General', 'General')
ON CONFLICT DO NOTHING;

INSERT INTO conversation_keys (id, conversation_id, user_id, user_role, encrypted_key) VALUES
  (rpad('ckey-1',24,'0'), rpad('conv-1',24,'0'), rpad('u-admin',24,'0'), 'admin', 'ENCRYPTED_KEY_PLACEHOLDER')
ON CONFLICT DO NOTHING;

INSERT INTO conversation_participants (id, conversation_id, school_id, user_id, role) VALUES
  (rpad('cpar-1',24,'0'), rpad('conv-1',24,'0'), rpad('s-main',24,'0'), rpad('u-admin',24,'0'), 'admin')
ON CONFLICT DO NOTHING;

INSERT INTO messages (id, conversation_id, school_id, sender_user_id, body, message_type) VALUES
  (rpad('msg-1',24,'0'), rpad('conv-1',24,'0'), rpad('s-main',24,'0'), rpad('u-admin',24,'0'), 'Welcome to the demo school!', 'text')
ON CONFLICT DO NOTHING;

INSERT INTO message_attachments (id, message_id, url, file_name, mime_type) VALUES
  (rpad('matt-1',24,'0'), rpad('msg-1',24,'0'), 'https://demo/file.pdf', 'file.pdf', 'application/pdf')
ON CONFLICT DO NOTHING;

INSERT INTO message_read_receipts (id, message_id, actor_id, actor_role) VALUES
  (rpad('mread-1',24,'0'), rpad('msg-1',24,'0'), rpad('u-student',24,'0'), 'student')
ON CONFLICT DO NOTHING;

-- ---------- social ----------
INSERT INTO announcements (id, school_id, class_id, author_user_id, title, body) VALUES
  (rpad('ann-1',24,'0'), rpad('s-main',24,'0'), rpad('cls-1',24,'0'), rpad('u-admin',24,'0'), 'Welcome', 'School opens September 1st.')
ON CONFLICT DO NOTHING;

INSERT INTO announcement_classes (announcement_id, class_id) VALUES
  (rpad('ann-1',24,'0'), rpad('cls-1',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO announcement_mentions (announcement_id, actor_id, actor_role) VALUES
  (rpad('ann-1',24,'0'), rpad('u-student',24,'0'), 'student')
ON CONFLICT DO NOTHING;

INSERT INTO comments (id, school_id, target_type, target_id, author_user_id, body, content) VALUES
  (rpad('cmt-1',24,'0'), rpad('s-main',24,'0'), 'announcement', rpad('ann-1',24,'0'), rpad('u-student',24,'0'), 'Thanks!', 'Thanks!')
ON CONFLICT DO NOTHING;

INSERT INTO likes (id, school_id, target_type, target_id, user_id) VALUES
  (rpad('like-1',24,'0'), rpad('s-main',24,'0'), 'announcement', rpad('ann-1',24,'0'), rpad('u-student',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO learning_materials (id, school_id, class_id, class_subject_id, subject_id, uploaded_by, title, description, material_type, url) VALUES
  (rpad('lm-1',24,'0'), rpad('s-main',24,'0'), rpad('cls-1',24,'0'), rpad('csub-1',24,'0'), rpad('csub-1',24,'0'), rpad('u-teacher',24,'0'), 'Algebra Notes', 'Chapter 1 notes', 'document', 'https://demo/notes.pdf')
ON CONFLICT DO NOTHING;

-- ---------- finance ----------
INSERT INTO finance_records (id, school_id, student_id, enrollment_id, user_id, record_type, description, amount, created_by) VALUES
  (rpad('fin-1',24,'0'), rpad('s-main',24,'0'), rpad('sp-1',24,'0'), rpad('enr-1',24,'0'), rpad('u-parent',24,'0'), 'tuition', 'Term 1 tuition', 50000, rpad('u-admin',24,'0'))
ON CONFLICT DO NOTHING;

-- ---------- links / memberships / access ----------
INSERT INTO parent_student_links (id, parent_id, parent_user_id, student_id, school_id, relationship) VALUES
  (rpad('psl-1',24,'0'), rpad('par-1',24,'0'), rpad('u-parent',24,'0'), rpad('sp-1',24,'0'), rpad('s-main',24,'0'), 'father')
ON CONFLICT DO NOTHING;

INSERT INTO school_memberships (id, school_id, user_id, member_type) VALUES
  (rpad('smem-1',24,'0'), rpad('s-main',24,'0'), rpad('u-admin',24,'0'), 'admin')
ON CONFLICT DO NOTHING;

INSERT INTO user_role_assignments (id, user_id, school_id, role, role_id) VALUES
  (rpad('ura-1',24,'0'), rpad('u-admin',24,'0'), rpad('s-main',24,'0'), 'admin', rpad('role-1',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO school_user_accessible_classes (id, school_id, user_id, class_id, granted_by) VALUES
  (rpad('suac-1',24,'0'), rpad('s-main',24,'0'), rpad('u-teacher',24,'0'), rpad('cls-1',24,'0'), rpad('u-admin',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO student_record_permissions (id, student_id, owner_school_id, viewer_school_id, scope, requested_by, granted_by) VALUES
  (rpad('srp-1',24,'0'), rpad('sp-1',24,'0'), rpad('s-main',24,'0'), rpad('s-main',24,'0'), 'academics', rpad('u-admin',24,'0'), rpad('u-admin',24,'0'))
ON CONFLICT DO NOTHING;

-- ---------- mongo import support ----------
INSERT INTO mongo_import_runs (id, source_main_db_name) VALUES
  (rpad('mir-1',24,'0'), 'legacy_main_db')
ON CONFLICT DO NOTHING;

INSERT INTO mongo_id_map (id, import_run_id, source_database_name, source_collection_name, source_id, target_table_name, target_id, school_id) VALUES
  (rpad('mim-1',24,'0'), rpad('mir-1',24,'0'), 'legacy_db', 'users', 'legacy_user_1', 'users', rpad('u-admin',24,'0'), rpad('s-main',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO student_identity_review_items (id, import_run_id, source_database_name, source_student_id, candidate_student_id, school_id, reason) VALUES
  (rpad('siri-1',24,'0'), rpad('mir-1',24,'0'), 'legacy_db', 'legacy_student_1', rpad('sp-1',24,'0'), rpad('s-main',24,'0'), 'possible duplicate')
ON CONFLICT DO NOTHING;

-- ---------- school config / profile ----------
INSERT INTO school_backups (id, school_id, backup_name, created_by) VALUES
  (rpad('bkp-1',24,'0'), rpad('s-main',24,'0'), 'Initial backup', rpad('u-admin',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO school_curricula (id, school_id, curriculum_id) VALUES
  (rpad('scur-1',24,'0'), rpad('s-main',24,'0'), rpad('cur-1',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO school_education_levels (id, school_id, education_level_id) VALUES
  (rpad('sel-1',24,'0'), rpad('s-main',24,'0'), rpad('elvl-1',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO school_feature_flags (id, school_id, feature_name, enabled) VALUES
  (rpad('sff-1',24,'0'), rpad('s-main',24,'0'), 'messaging', true)
ON CONFLICT DO NOTHING;

INSERT INTO school_features (school_id, feature_name, enabled) VALUES
  (rpad('s-main',24,'0'), 'attendance', true)
ON CONFLICT DO NOTHING;

INSERT INTO school_profile_values (id, school_id, value_type, value) VALUES
  (rpad('spv-1',24,'0'), rpad('s-main',24,'0'), 'motto', 'Learn Together')
ON CONFLICT DO NOTHING;

INSERT INTO school_social_media (id, school_id, platform, url) VALUES
  (rpad('ssm-1',24,'0'), rpad('s-main',24,'0'), 'twitter', 'https://twitter.com/demoschool')
ON CONFLICT DO NOTHING;

INSERT INTO school_staff_tags (id, staff_id, tag) VALUES
  (rpad('sst-1',24,'0'), rpad('stf-1',24,'0'), 'reception')
ON CONFLICT DO NOTHING;

INSERT INTO school_timetables (id, school_id, academic_year_id) VALUES
  (rpad('stt-1',24,'0'), rpad('s-main',24,'0'), rpad('eyear-1',24,'0'))
ON CONFLICT DO NOTHING;

-- ---------- class config ----------
INSERT INTO class_settings (class_id, student_can_chat, teacher_can_take_attendance) VALUES
  (rpad('cls-1',24,'0'), true, true)
ON CONFLICT DO NOTHING;

INSERT INTO class_background_images (id, class_id, public_id, url) VALUES
  (rpad('cbg-1',24,'0'), rpad('cls-1',24,'0'), 'bg_public_1', 'https://demo/bg.jpg')
ON CONFLICT DO NOTHING;

INSERT INTO class_break_times (id, class_id, start_time, end_time, label) VALUES
  (rpad('cbt-1',24,'0'), rpad('cls-1',24,'0'), '10:00', '10:20', 'Morning break')
ON CONFLICT DO NOTHING;

INSERT INTO class_subject_topics (id, class_subject_id, order_key, title, description) VALUES
  (rpad('cstp-1',24,'0'), rpad('csub-1',24,'0'), '01', 'Introduction to Algebra', 'Basics')
ON CONFLICT DO NOTHING;

INSERT INTO class_tags (id, class_id, tag) VALUES
  (rpad('ctag-1',24,'0'), rpad('cls-1',24,'0'), 'morning')
ON CONFLICT DO NOTHING;

INSERT INTO class_timetable_periods (id, class_id, day_key, period, subject, teacher_id) VALUES
  (rpad('ctp-1',24,'0'), rpad('cls-1',24,'0'), 'monday', 1, 'Mathematics', rpad('tch-1',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO class_timetables (id, class_id, education_year_id) VALUES
  (rpad('ctt-1',24,'0'), rpad('cls-1',24,'0'), rpad('eyear-1',24,'0'))
ON CONFLICT DO NOTHING;

-- ---------- teacher / student / user link rows ----------
INSERT INTO teacher_classes (id, teacher_id, class_id, school_id) VALUES
  (rpad('tcl-1',24,'0'), rpad('tch-1',24,'0'), rpad('cls-1',24,'0'), rpad('s-main',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO teacher_subjects (id, teacher_id, class_subject_id, school_id) VALUES
  (rpad('tsub-1',24,'0'), rpad('tch-1',24,'0'), rpad('csub-1',24,'0'), rpad('s-main',24,'0'))
ON CONFLICT DO NOTHING;

INSERT INTO teacher_tags (id, teacher_id, tag) VALUES
  (rpad('ttag-1',24,'0'), rpad('tch-1',24,'0'), 'senior')
ON CONFLICT DO NOTHING;

INSERT INTO student_enrollment_tags (id, enrollment_id, tag) VALUES
  (rpad('set-1',24,'0'), rpad('enr-1',24,'0'), 'scholarship')
ON CONFLICT DO NOTHING;

INSERT INTO user_background_images (id, user_id, image_id, url) VALUES
  (rpad('ubg-1',24,'0'), rpad('u-admin',24,'0'), 'ubg_public_1', 'https://demo/userbg.jpg')
ON CONFLICT DO NOTHING;

INSERT INTO user_profile_values (id, user_id, value_type, value) VALUES
  (rpad('upv-1',24,'0'), rpad('u-admin',24,'0'), 'title', 'Principal')
ON CONFLICT DO NOTHING;

INSERT INTO user_public_keys (id, user_id, public_key) VALUES
  (rpad('upk-1',24,'0'), rpad('u-admin',24,'0'), 'PUBLIC_KEY_PLACEHOLDER')
ON CONFLICT DO NOTHING;

INSERT INTO user_social_media (id, user_id, platform, url) VALUES
  (rpad('usm-1',24,'0'), rpad('u-admin',24,'0'), 'linkedin', 'https://linkedin.com/in/demoadmin')
ON CONFLICT DO NOTHING;

-- ---------- requests / audit ----------
INSERT INTO join_school_requests (id, school_id, invited_user_id, class_id, role, email, sent_by, message) VALUES
  (rpad('jsr-1',24,'0'), rpad('s-main',24,'0'), rpad('u-teacher',24,'0'), rpad('cls-1',24,'0'), 'teacher', 'teacher@demo.school', rpad('u-admin',24,'0'), 'Please join')
ON CONFLICT DO NOTHING;

INSERT INTO audit_logs (id, school_id, actor_user_id, entity_type, entity_id, action) VALUES
  (rpad('alog-1',24,'0'), rpad('s-main',24,'0'), rpad('u-admin',24,'0'), 'school', rpad('s-main',24,'0'), 'seed')
ON CONFLICT DO NOTHING;

COMMIT;
