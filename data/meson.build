# SPDX-License-Identifier: GPL-3.0-or-later
conf = configuration_data()
conf.set('APP_ID', app_id)
conf.set('WEBSITE', website)
conf.set('REPOSITORY', repository)
conf.set('VERSION', meson.project_version())
po_dir = meson.project_source_root() / 'po'

desktop_file = i18n.merge_file(
  input: configure_file(
    input: 'trigonum.desktop.in.in',
    output: app_id + '.desktop.in',
    configuration: conf,
  ),
  output: app_id + '.desktop',
  type: 'desktop',
  po_dir: po_dir,
  install: true,
  install_dir: get_option('datadir') / 'applications',
)

metainfo_file = i18n.merge_file(
  input: configure_file(
    input: 'trigonum.metainfo.xml.in.in',
    output: app_id + '.metainfo.xml.in',
    configuration: conf,
  ),
  output: app_id + '.metainfo.xml',
  po_dir: po_dir,
  install: true,
  install_dir: get_option('datadir') / 'metainfo',
)

install_data(
  'icons/trigonum.svg',
  install_dir: get_option('datadir') / 'icons' / 'hicolor' / 'scalable' / 'apps',
  rename: app_id + '.svg',
)

desktop_validate = find_program('desktop-file-validate', required: false)
if desktop_validate.found()
  test('validate-desktop', desktop_validate, args: [desktop_file])
endif

appstreamcli = find_program('appstreamcli', required: false)
if appstreamcli.found()
  test('validate-metainfo', appstreamcli, args: ['validate', '--no-net', '--explain', metainfo_file])
endif
