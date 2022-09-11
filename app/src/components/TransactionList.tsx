import { useState } from 'react';
import { Row, Col, Tabs, Button, Form, Select } from 'antd';
import { MultisigWallet } from '../types';

interface Props {
  multisigWallet: MultisigWallet
  onApprove: any
  onExecute: any
};

export const TransactionList: React.FC<Props> = ({
  multisigWallet,
  onApprove,
  onExecute,
}) => {
  const [loading, setLoading] = useState(false)
  if (!multisigWallet.transactions || multisigWallet.transactions.length <= 0) {
    return (<></>);
  }
  return (
    <Row>
      <Col span={24}>
        <Tabs defaultActiveKey="1">
          {
            multisigWallet.transactions.map((transaction, idx) => (
              <Tabs.TabPane tab={`Transaction ${idx+1}`} key={idx}>
                <Row>
                  <Col span={24}>
                    <p><b>Transaction data:</b> {transaction.data}</p>
                    <p><b>Signers:</b> {transaction.signers.length}</p>
                  </Col>
                  <Col span={24}>
                    <Form
                      onFinish={async (values) => {
                        try {
                          setLoading(true)
                          await onApprove(multisigWallet, transaction, values.approver)
                        } catch (e) {}
                        setLoading(false)
                      }}
                    >
                      { transaction.signers.length < multisigWallet.threshold ? (
                        <Row gutter={10}>
                          <Col span={20}>
                            <Form.Item
                              name="approver"
                              label="Approver"
                              rules={[
                                { required: true },
                              ]}
                            >
                              <Select
                                placeholder="Approver"
                                options={multisigWallet.owners
                                  .filter(
                                    owner => transaction.proposer.publicKey.toString() !== owner.toString()
                                  ).map(owner => ({
                                  label: owner.toString(),
                                  value: owner.toString(),
                                }))}
                              />
                            </Form.Item>
                          </Col>
                          <Col span={2}>
                            <Form.Item>
                              <Button type="primary" htmlType="submit" loading={loading}>
                                Approve
                              </Button>
                            </Form.Item>
                          </Col>
                        </Row>
                      ) : !transaction.executed ? (
                        <Button
                          type="primary"
                          loading={loading}
                          onClick={async () => {
                            try {
                              setLoading(true)
                              await onExecute(multisigWallet, transaction)
                            } catch (e) {
                              console.error(e)
                            }
                            setLoading(false)
                          }}
                        >
                          Execute
                        </Button>
                      ) : (
                        <p>Transaction already executed.</p>
                      )}
                    </Form>
                  </Col>
                </Row>
              </Tabs.TabPane>
            ))
          }
        </Tabs>
      </Col>
    </Row>
  );
};
