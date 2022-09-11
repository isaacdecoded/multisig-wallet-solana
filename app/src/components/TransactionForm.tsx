import React, { useState } from 'react';
import { Button, Form, Input, Select } from 'antd';
import { MultisigWallet } from '../types';

const formItemLayoutWithOutLabel = {
  wrapperCol: {
    xs: { span: 24, offset: 0 },
    sm: { span: 20, offset: 4 },
  },
};

interface Props {
  multisigWallet: MultisigWallet
  onCreate: any
};

const formItemLayout = {
  labelCol: {
    xs: { span: 24 },
    sm: { span: 4 },
  },
  wrapperCol: {
    xs: { span: 24 },
    sm: { span: 20 },
  },
};

export const TransactionForm: React.FC<Props> = ({
  multisigWallet,
  onCreate,
}) => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false)

  return (
    <Form
      form={form}
      {...formItemLayoutWithOutLabel}
      onFinish={async (values) => {
        try {
          setLoading(true)
          await onCreate({
            ...values,
            multisigWallet,
          });
          form.resetFields();
        } catch (e) {}
        setLoading(false)
      }}
    >
      <Form.Item
        name="proposer"
        label="Proposer"
        rules={[
          { required: true },
        ]}
        {...formItemLayout}
      >
        <Select
          placeholder="Proposer"
          options={multisigWallet.owners.map(owner => ({
            label: owner.toString(),
            value: owner.toString(),
          }))}
        />
      </Form.Item>
      <Form.Item
        name="data"
        label="Data"
        rules={[
          { required: true },
        ]}
        {...formItemLayout}
      >
        <Input placeholder="Data" />
      </Form.Item>
      <Form.Item>
        <Button type="primary" htmlType="submit" loading={loading}>
          Create
        </Button>
      </Form.Item>
    </Form>
  )
};
