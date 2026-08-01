const { createApp } = Vue;

createApp({
    data() {
        return {
            currentTab: 'create',
            tests: [],
            currentLogs: [],
            showLogsModal: false,
            form: {
                engine1_name: '',
                engine1_ref: '',
                engine2_name: '',
                engine2_ref: '',
                env_vars_str: '{}',
                fastchess_params_str: '{}',
                discord_webhook: '',
            },
            settings: {
                compiled_engines_path: '',
                lora_repo_path: '',
                fastchess_path: '',
                default_env_vars_str: '{}',
                default_fastchess_params_str: '{}',
                default_discord_webhook: '',
            },
            createError: '',
            settingsError: '',
            settingsSuccess: false,
        };
    },
    mounted() {
        this.loadSettings();
        this.loadTests();
        // Auto-refresh tests every 5 seconds
        setInterval(() => this.loadTests(), 5000);
    },
    methods: {
        async createTest() {
            this.createError = '';
            try {
                const envVars = JSON.parse(this.form.env_vars_str);
                const fastchessParams = JSON.parse(this.form.fastchess_params_str);

                const response = await fetch('/api/tests', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        engine1_name: this.form.engine1_name,
                        engine1_ref: this.form.engine1_ref,
                        engine2_name: this.form.engine2_name,
                        engine2_ref: this.form.engine2_ref,
                        env_vars: envVars,
                        fastchess_params: fastchessParams,
                        discord_webhook: this.form.discord_webhook || null,
                    }),
                });

                if (!response.ok) {
                    throw new Error('Failed to create test');
                }

                // Reset form
                this.form = {
                    engine1_name: '',
                    engine1_ref: '',
                    engine2_name: '',
                    engine2_ref: '',
                    env_vars_str: '{}',
                    fastchess_params_str: '{}',
                    discord_webhook: '',
                };

                this.currentTab = 'running';
                this.loadTests();
            } catch (error) {
                this.createError = error.message || 'Failed to create test';
            }
        },

        async loadTests() {
            try {
                const response = await fetch('/api/tests');
                this.tests = await response.json();
            } catch (error) {
                console.error('Failed to load tests:', error);
            }
        },

        async pauseTest(testId) {
            try {
                await fetch(`/api/tests/${testId}/pause`, { method: 'POST' });
                this.loadTests();
            } catch (error) {
                console.error('Failed to pause test:', error);
            }
        },

        async resumeTest(testId) {
            try {
                await fetch(`/api/tests/${testId}/resume`, { method: 'POST' });
                this.loadTests();
            } catch (error) {
                console.error('Failed to resume test:', error);
            }
        },

        async deleteTest(testId) {
            if (confirm('Are you sure you want to discard this test?')) {
                try {
                    await fetch(`/api/tests/${testId}`, { method: 'DELETE' });
                    this.loadTests();
                } catch (error) {
                    console.error('Failed to delete test:', error);
                }
            }
        },

        async viewLogs(testId) {
            try {
                const response = await fetch(`/api/tests/${testId}/logs`);
                this.currentLogs = await response.json();
                this.showLogsModal = true;
            } catch (error) {
                console.error('Failed to load logs:', error);
            }
        },

        async loadSettings() {
            try {
                const response = await fetch('/api/settings');
                const data = await response.json();
                this.settings = {
                    compiled_engines_path: data.compiled_engines_path,
                    lora_repo_path: data.lora_repo_path,
                    fastchess_path: data.fastchess_path,
                    default_env_vars_str: JSON.stringify(data.default_env_vars, null, 2),
                    default_fastchess_params_str: JSON.stringify(data.default_fastchess_params, null, 2),
                    default_discord_webhook: data.default_discord_webhook || '',
                };
            } catch (error) {
                console.error('Failed to load settings:', error);
            }
        },

        async saveSettings() {
            this.settingsError = '';
            this.settingsSuccess = false;
            try {
                const response = await fetch('/api/settings', {
                    method: 'PUT',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        default_env_vars: JSON.parse(this.settings.default_env_vars_str),
                        default_fastchess_params: JSON.parse(this.settings.default_fastchess_params_str),
                        compiled_engines_path: this.settings.compiled_engines_path,
                        lora_repo_path: this.settings.lora_repo_path,
                        fastchess_path: this.settings.fastchess_path,
                        default_discord_webhook: this.settings.default_discord_webhook || null,
                    }),
                });

                if (!response.ok) {
                    throw new Error('Failed to save settings');
                }

                this.settingsSuccess = true;
                setTimeout(() => { this.settingsSuccess = false; }, 3000);
            } catch (error) {
                this.settingsError = error.message || 'Failed to save settings';
            }
        },

        formatDate(dateStr) {
            try {
                return new Date(dateStr).toLocaleString();
            } catch {
                return dateStr;
            }
        },
    },
}).mount('#app');
